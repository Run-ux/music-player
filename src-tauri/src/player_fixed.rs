use std::fs::File;
use std::io::BufReader;
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

                SongInfo {
                    path: path_str,
                    title: tag.title().map(|s| s.to_string()),
                    artist: tag.artist().map(|s| s.to_string()),
                    album: tag.album().map(|s| s.to_string()),
                    album_cover,
                    duration: None,
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

        // 尝试获取音频时长（需要打开文件并解码以获取时长）
        if let Ok(file) = File::open(path) {
            // rodio::Source is needed for total_duration
            // Ensure rodio::Source is available or find alternative for duration
            if let Ok(source) = rodio::Decoder::new(BufReader::new(file)) {
                // rodio::Source trait is needed for total_duration()
                use rodio::Source;
                song_info.duration = source.total_duration().map(|d| d.as_secs());
            }
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
    Stop,
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
