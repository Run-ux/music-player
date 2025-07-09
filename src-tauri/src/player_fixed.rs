use std::fs::File;
use std::io::{BufReader};
use std::path::Path; // 移除未使用的PathBuf导入

use anyhow::Result;
use base64::Engine;
use id3::{Tag, TagLike};
use image::{ImageFormat, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;

/// 音乐播放器错误类型
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum PlayerError {
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("解码错误: {0}")]
    DecoderError(#[from] rodio::decoder::DecoderError),

    #[error("播放列表为空")]
    EmptyPlaylist,

    #[error("无效的歌曲索引")]
    InvalidSongIndex,

    #[error("ID3标签错误: {0}")]
    Id3Error(#[from] id3::Error),

    #[error("音频播放错误: {0}")]
    PlayError(String), // Changed from rodio::PlayError to String if direct rodio errors are not bubbled up

    #[error("其他错误: {0}")]
    OtherError(#[from] anyhow::Error),
}

/// 播放模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMode {
    Sequential, // 顺序播放
    Repeat,     // 单曲循环
    Shuffle,    // 随机播放
}

/// 播放器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

/// 歌曲信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfo {
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    #[serde(rename = "albumCover")]
    pub album_cover: Option<String>,
    pub duration: Option<u64>, // 单位：秒
}

impl SongInfo {    /// 从文件路径创建歌曲信息
    pub fn from_path(path: &Path) -> Result<Self> {
        let path_str = path.to_string_lossy().into_owned();
        
        let mut song_info = match Tag::read_from_path(path) {
            Ok(tag) => {
                // 提取专辑封面
                let album_cover = Self::extract_album_cover(&tag);
                
                // 尝试从ID3标签获取时长
                let duration = tag.duration().map(|d| d as u64);

                SongInfo {
                    path: path_str,
                    title: tag.title().map(|s| s.to_string()),
                    artist: tag.artist().map(|s| s.to_string()),
                    album: tag.album().map(|s| s.to_string()),
                    album_cover,
                    duration,
                }
            }
            Err(_) => {
                // 如果无法读取标签，则使用文件名作为标题，并设置默认封面
                SongInfo {
                    path: path_str,
                    title: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                    artist: None,
                    album: None,
                    album_cover: Self::get_default_album_cover(),
                    duration: None,
                }
            }
        };

        // 如果标签中没有时长信息，尝试从文件解码获取
        if song_info.duration.is_none() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            // 使用更准确的时长获取方法
            song_info.duration = Self::get_accurate_duration(path, &ext);
        }
        
        Ok(song_info)
    }    /// 从ID3标签提取专辑封面
    fn extract_album_cover(tag: &Tag) -> Option<String> {
        // 查找第一个图片
        let pictures: Vec<_> = tag.pictures().collect();

        if let Some(picture) = pictures.first() {
            match Self::convert_image_to_base64(&picture.data) {
                Ok(base64_string) => {
                    // 根据图片类型确定MIME类型
                    let mime_type = match picture.mime_type.as_str() {
                        "image/jpeg" => "image/jpeg",
                        "image/png" => "image/png",
                        _ => "image/jpeg", // 默认使用JPEG
                    };
                    let data_url = format!("data:{};base64,{}", mime_type, base64_string);
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else {
            // 没有找到专辑封面，使用默认封面
            Self::get_default_album_cover()
        }
    }    /// 获取默认专辑封面
    fn get_default_album_cover() -> Option<String> {
        // 尝试从多个可能的默认图片路径生成Base64
        let possible_paths = [
            "src/assets/default-cover.jpg",
            "../src/assets/default-cover.jpg",
            "../../src/assets/default-cover.jpg",
            "assets/default-cover.jpg",
            "default-cover.jpg",
        ];

        for path in &possible_paths {
            match std::fs::read(path) {
                Ok(image_data) => {
                    match Self::convert_image_to_base64(&image_data) {
                        Ok(base64_string) => {
                            let data_url = format!("data:image/jpeg;base64,{}", base64_string);
                            return Some(data_url);
                        }
                        Err(_) => {
                            // 转换失败，继续尝试下一个文件
                        }
                    }
                }
                Err(_) => {
                    // 继续尝试下一个路径
                    continue;
                }
            }
        }

        // 如果无法读取任何文件，返回一个简单的颜色块作为默认封面
        Self::generate_fallback_cover()
    }    /// 生成一个简单的颜色块作为默认封面
    fn generate_fallback_cover() -> Option<String> {
        // 创建一个300x300的简单图片
        let mut img = RgbImage::new(300, 300);

        // 填充渐变色
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = (x as f32 / 300.0 * 100.0 + 100.0) as u8;
            let g = (y as f32 / 300.0 * 100.0 + 100.0) as u8;
            let b = 150u8;
            *pixel = Rgb([r, g, b]);
        }

        // 转换为JPEG格式
        let mut jpeg_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_bytes);

        match img.write_to(&mut cursor, ImageFormat::Jpeg) {
            Ok(_) => {
                let base64_string = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);
                let data_url = format!("data:image/jpeg;base64,{}", base64_string);
                Some(data_url)
            }
            Err(_) => None,
        }
    }    /// 将图片数据转换为Base64字符串
    fn convert_image_to_base64(image_data: &[u8]) -> Result<String> {
        // 尝试加载图片
        let img = image::load_from_memory(image_data)?;

        // 调整图片大小以减少数据量（最大300x300）
        let resized_img = img.resize(300, 300, image::imageops::FilterType::Lanczos3);

        // 转换为JPEG格式
        let mut jpeg_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_bytes);
        resized_img.write_to(&mut cursor, ImageFormat::Jpeg)?;

        // 编码为Base64
        let base64_string = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);

        Ok(base64_string)
    }    /// 获取OGG文件时长（专门针对OGG文件的解析方法）
    #[allow(dead_code)]
    fn get_ogg_duration(path: &Path) -> Option<u64> {
        use std::io::Read;

        // 简化的OGG时长获取，避免复杂的页面解析
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };

        let mut header = [0; 512];
        if file.read(&mut header).is_err() {
            return None;
        }

        // 检查是否为OGG文件
        if &header[0..4] != b"OggS" {
            return None;
        }

        // 对于OGG文件，我们使用文件大小估算
        if let Ok(metadata) = std::fs::metadata(path) {
            let file_size_bytes = metadata.len() as f64;
            // OGG Vorbis 通常使用较低的比特率
            let bitrate = 128000.0; // 128 kbps
            let estimated_seconds = (file_size_bytes / (bitrate / 8.0)).round() as u64;
            
            if estimated_seconds > 0 && estimated_seconds < 10800 {
                return Some(estimated_seconds);
            }
        }

        None
    }

    /// 获取文件的准确时长（支持多种音频格式）
    fn get_accurate_duration(path: &Path, ext: &str) -> Option<u64> {
        println!("正在获取文件时长: {}", path.display());
        
        // 方法1: 使用rodio解码器获取时长（最准确）
        if let Some(duration) = Self::try_rodio_duration(path) {
            println!("通过rodio获取到时长: {}秒", duration);
            return Some(duration);
        }
        
        // 方法2: 针对不同格式使用专门的解析方法
        let duration = match ext {
            "ogg" => Self::get_ogg_duration_advanced(path),
            "mp3" => Self::get_mp3_duration(path),
            "flac" => Self::get_flac_duration(path),
            "wav" => Self::get_wav_duration(path),
            "m4a" | "aac" => Self::get_aac_duration(path),
            _ => None,
        };
        
        if let Some(d) = duration {
            println!("通过格式特定方法获取到时长: {}秒", d);
            return Some(d);
        }
        
        // 方法3: 使用文件大小估算（最后的兜底方案）
        let estimated = Self::estimate_duration_from_filesize(path, ext);
        if let Some(d) = estimated {
            println!("通过文件大小估算时长: {}秒", d);
        }
        
        estimated
    }
    
    /// 尝试使用rodio解码器获取时长
    fn try_rodio_duration(path: &Path) -> Option<u64> {
        // 尝试多次，有时第一次会失败
        for attempt in 0..3 {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(source) = rodio::Decoder::new(reader) {
                    use rodio::Source;
                    if let Some(total_duration) = source.total_duration() {
                        let seconds = total_duration.as_secs();
                        // 有效性检查
                        if seconds > 0 && seconds < 10800 { // 0-3小时范围
                            return Some(seconds);
                        }
                    }
                }
            }
            
            // 如果失败，稍等一下再试
            if attempt < 2 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        None
    }
    
    /// 高级OGG时长获取方法
    fn get_ogg_duration_advanced(path: &Path) -> Option<u64> {
        use std::io::{Read, Seek, SeekFrom};
        
        let mut file = File::open(path).ok()?;
        let mut buffer = vec![0u8; 1024];
        
        // 读取文件头
        file.read_exact(&mut buffer[..27]).ok()?;
        
        // 检查OGG签名
        if &buffer[0..4] != b"OggS" {
            return None;
        }
        
        // 尝试查找最后一个OGG页面来获取总时长
        let file_size = file.metadata().ok()?.len();
        if file_size < 1024 {
            return Self::estimate_duration_from_filesize(path, "ogg");
        }
        
        // 从文件末尾开始查找
        let search_pos = file_size.saturating_sub(65536).max(0);
        file.seek(SeekFrom::Start(search_pos)).ok()?;
        
        let mut last_buffer = vec![0u8; 65536];
        let bytes_read = file.read(&mut last_buffer).ok()?;
        
        // 在缓冲区中查找最后的OGG页面
        for i in (0..bytes_read.saturating_sub(27)).rev() {
            if &last_buffer[i..i+4] == b"OggS" {
                // 尝试解析granule position (字节14-21)
                if i + 21 < bytes_read {
                    let granule_pos = u64::from_le_bytes([
                        last_buffer[i+6], last_buffer[i+7], last_buffer[i+8], last_buffer[i+9],
                        last_buffer[i+10], last_buffer[i+11], last_buffer[i+12], last_buffer[i+13]
                    ]);
                    
                    if granule_pos > 0 && granule_pos < 0xFFFFFFFFFFFFFFFF {
                        // 对于Vorbis，granule position通常是采样数
                        // 假设采样率为44100 Hz（CD质量）
                        let duration_seconds = granule_pos / 44100;
                        if duration_seconds > 0 && duration_seconds < 10800 {
                            return Some(duration_seconds);
                        }
                    }
                }
            }
        }
        
        // 如果无法解析页面，使用文件大小估算
        Self::estimate_duration_from_filesize(path, "ogg")
    }
    
    /// MP3时长获取
    fn get_mp3_duration(path: &Path) -> Option<u64> {
        // 对于MP3，先尝试使用ID3标签，然后使用文件大小估算
        if let Ok(tag) = Tag::read_from_path(path) {
            if let Some(duration) = tag.duration() {
                return Some(duration as u64);
            }
        }
        Self::estimate_duration_from_filesize(path, "mp3")
    }
    
    /// FLAC时长获取
    fn get_flac_duration(path: &Path) -> Option<u64> {
        use std::io::Read;
        
        let mut file = File::open(path).ok()?;
        let mut header = [0u8; 4];
        file.read_exact(&mut header).ok()?;
        
        // 检查FLAC签名
        if &header != b"fLaC" {
            return None;
        }
        
        // FLAC的metadata通常在文件开头，这里使用估算
        Self::estimate_duration_from_filesize(path, "flac")
    }
    
    /// WAV时长获取
    fn get_wav_duration(path: &Path) -> Option<u64> {
        use std::io::Read;
        
        let mut file = File::open(path).ok()?;
        let mut header = [0u8; 44]; // WAV header通常是44字节
        
        if file.read_exact(&mut header).is_err() {
            return None;
        }
        
        // 检查WAV签名
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return None;
        }
        
        // 提取采样率和数据大小
        let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
        let byte_rate = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
        
        if sample_rate > 0 && byte_rate > 0 {
            let data_size = file.metadata().ok()?.len().saturating_sub(44);
            let duration_seconds = data_size / byte_rate as u64;
            
            if duration_seconds > 0 && duration_seconds < 10800 {
                return Some(duration_seconds);
            }
        }
        
        Self::estimate_duration_from_filesize(path, "wav")
    }
    
    /// AAC/M4A时长获取
    fn get_aac_duration(path: &Path) -> Option<u64> {
        // AAC/M4A格式比较复杂，这里使用估算
        Self::estimate_duration_from_filesize(path, "m4a")
    }
    
    /// 基于文件大小估算时长
    fn estimate_duration_from_filesize(path: &Path, ext: &str) -> Option<u64> {
        let metadata = std::fs::metadata(path).ok()?;
        let file_size_bytes = metadata.len() as f64;
        
        // 根据文件扩展名估计比特率（单位：比特每秒）
        let bitrate = match ext {
            "mp3" => 160000.0,      // 中等质量MP3
            "flac" => 850000.0,     // 无损格式，稍微保守一点
            "wav" => 1411200.0,     // 标准CD质量无压缩
            "ogg" => 112000.0,      // OGG Vorbis，稍微保守
            "m4a" | "aac" => 128000.0, // AAC格式
            "wma" => 128000.0,      // Windows Media Audio
            _ => 128000.0,          // 默认值
        };
        
        // 文件大小（字节） / (比特率（比特/秒） / 8) = 秒数
        let estimated_seconds = (file_size_bytes / (bitrate / 8.0)).round() as u64;
        
        // 有效性检查
        if estimated_seconds > 0 && estimated_seconds < 10800 { // 最长3小时
            Some(estimated_seconds)
        } else {
            println!("估算时长超出合理范围: {}秒", estimated_seconds);
            None
        }
    }
}

/// 播放器事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PlayerEvent {
    StateChanged(PlayerState),
    SongChanged(usize, SongInfo),
    PlaylistUpdated(Vec<SongInfo>),
    ProgressUpdate { position: u64, duration: u64 },
    Error(String),
}

/// 播放器命令
#[derive(Debug)] // Consider adding Clone if commands need to be cloned (e.g. for internal use)
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,            // 添加 Stop 命令
    Next,
    Previous,
    SetSong(usize),
    AddSong(SongInfo),       // Changed from PathBuf to SongInfo
    AddSongs(Vec<SongInfo>), // Added new variant for adding multiple songs
    RemoveSong(usize),
    ClearPlaylist,
    SetPlayMode(PlayMode),
    SetVolume(f32), // Added SetVolume command
    SeekTo(u64),    // Added SeekTo command (position in seconds)
}
