use std::fs::File;
use std::io::{BufReader};
use std::path::Path;

use anyhow::Result;
use base64::Engine;
use id3::{Tag, TagLike};
use image::{ImageFormat, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;
// 添加新的元数据库依赖
use lofty::{AudioFile, Probe, TaggedFileExt, Accessor};
use audiotags::Tag as AudioTag;

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

/// 歌词行结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub time: u64,      // 时间戳（毫秒）
    pub text: String,   // 歌词文本
}

/// 媒体类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
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
    pub lyrics: Option<Vec<LyricLine>>, // 歌词信息
    // 新增：MV相关字段
    #[serde(rename = "mediaType")]
    pub media_type: Option<MediaType>,  // 媒体类型
    #[serde(rename = "mvPath")]
    pub mv_path: Option<String>,        // MV视频文件路径
    #[serde(rename = "videoThumbnail")]
    pub video_thumbnail: Option<String>, // 视频缩略图
    #[serde(rename = "hasLyrics")]
    pub has_lyrics: Option<bool>,       // 是否有歌词
}

impl SongInfo {
    /// 从文件路径创建歌曲信息 - 使用四重兜底策略
    pub fn from_path(path: &Path) -> Result<Self> {
        let path_str = path.to_string_lossy().into_owned();
        println!("正在解析媒体文件: {}", path.display());
        
        // 检查文件扩展名确定媒体类型
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let media_type = if Self::is_video_format(&ext) {
            Some(MediaType::Video)
        } else if Self::is_audio_format(&ext) {
            Some(MediaType::Audio)
        } else {
            None
        };
        
        // 对于视频文件，使用特殊处理
        if media_type == Some(MediaType::Video) {
            return Self::create_video_song_info(path);
        }
        
        // 对于音频文件，继续使用原有逻辑
        // 策略1: 使用 lofty 库（最强大的通用库）
        if let Some(mut song_info) = Self::try_lofty_extraction(path) {
            println!("✅ 使用 lofty 库成功提取元数据");
            song_info.media_type = media_type;
            song_info.has_lyrics = Some(song_info.lyrics.is_some());
            // 尝试加载歌词
            song_info.lyrics = Self::load_lyrics(path);
            return Ok(song_info);
        }
        
        // 策略2: 使用 audiotags 库
        if let Some(mut song_info) = Self::try_audiotags_extraction(path) {
            println!("✅ 使用 audiotags 库成功提取元数据");
            song_info.media_type = media_type;
            song_info.has_lyrics = Some(song_info.lyrics.is_some());
            // 尝试加载歌词
            song_info.lyrics = Self::load_lyrics(path);
            return Ok(song_info);
        }
        
        // 策略3: 使用格式特定的方法（原有的 ID3/FLAC/OGG 方法）
        if let Some(mut song_info) = Self::try_format_specific_extraction(path) {
            println!("✅ 使用格式特定方法成功提取元数据");
            song_info.media_type = media_type;
            song_info.has_lyrics = Some(song_info.lyrics.is_some());
            // 尝试加载歌词
            song_info.lyrics = Self::load_lyrics(path);
            return Ok(song_info);
        }
        
        // 策略4: 兜底方案，使用文件名作为标题
        println!("⚠️  所有元数据提取方法都失败，使用兜底方案");
        let mut song_info = Self::create_fallback_song_info(path);
        song_info.media_type = media_type;
        song_info.has_lyrics = Some(song_info.lyrics.is_some());
        // 尝试加载歌词
        song_info.lyrics = Self::load_lyrics(path);
        Ok(song_info)
    }

    /// 检查是否为视频格式
    fn is_video_format(ext: &str) -> bool {
        matches!(ext, "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v")
    }

    /// 检查是否为音频格式
    fn is_audio_format(ext: &str) -> bool {
        matches!(ext, "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "wma")
    }

    /// 创建视频文件信息
    fn create_video_song_info(path: &Path) -> Result<Self> {
        let path_str = path.to_string_lossy().into_owned();
        println!("正在处理视频文件: {}", path.display());
        
        // 提取文件名作为标题
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        
        // 对于视频文件，不估算时长，让前端VideoPlayer来提供真实时长
        let duration = None;
        
        // 尝试生成视频缩略图
        let video_thumbnail = Self::generate_video_thumbnail(path);
        
        // 检查是否有对应的歌词文件
        let lyrics = Self::load_lyrics(path);
        
        Ok(SongInfo {
            path: path_str.clone(),
            title,
            artist: None, // 视频文件通常没有艺术家信息
            album: None,  // 视频文件通常没有专辑信息
            album_cover: video_thumbnail.clone(), // 使用视频缩略图作为封面
            duration, // 设置为None，由前端提供真实时长
            lyrics: lyrics.clone(),
            media_type: Some(MediaType::Video),
            mv_path: Some(path_str), // MV路径就是文件本身的路径
            video_thumbnail,
            has_lyrics: Some(lyrics.is_some()),
        })
    }

    /// 生成视频缩略图
    fn generate_video_thumbnail(_path: &Path) -> Option<String> {
        // 这里可以使用ffmpeg来生成视频缩略图
        // 目前先返回None，后续可以改进
        // 可以生成一个默认的视频图标
        Self::generate_video_placeholder()
    }

    /// 生成视频占位图
    fn generate_video_placeholder() -> Option<String> {
        // 创建一个300x300的视频占位图
        let mut img = RgbImage::new(300, 300);

        // 填充深色背景
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = 40u8;
            let g = 40u8;
            let b = 50u8;
            *pixel = Rgb([r, g, b]);
        }

        // 在中心画一个播放按钮（简单的三角形）
        let center_x = 150i32;
        let center_y = 150i32;
        let triangle_size = 40i32;

        for y in (center_y - triangle_size)..(center_y + triangle_size) {
            for x in (center_x - triangle_size)..(center_x + triangle_size) {
                if x >= 0 && y >= 0 && (x as u32) < 300 && (y as u32) < 300 {
                    // 简单的三角形形状判断
                    let dx = x - center_x;
                    let dy = y - center_y;
                    
                    if dx > -triangle_size / 2 && dx < triangle_size / 2 &&
                       dy.abs() < triangle_size - dx.abs() / 2 && dx > -dy.abs() / 2 {
                        img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255])); // 白色播放按钮
                    }
                }
            }
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
    }

    /// 加载歌词文件
    fn load_lyrics(audio_path: &Path) -> Option<Vec<LyricLine>> {
        let audio_dir = audio_path.parent()?;
        let audio_stem = audio_path.file_stem()?.to_str()?;
        
        // 可能的歌词文件扩展名
        let lyric_extensions = ["lrc", "txt"];
        
        for ext in &lyric_extensions {
            let lyric_path = audio_dir.join(format!("{}.{}", audio_stem, ext));
            
            if lyric_path.exists() {
                println!("找到歌词文件: {}", lyric_path.display());
                
                match ext {
                    &"lrc" => {
                        if let Some(lyrics) = Self::parse_lrc_file(&lyric_path) {
                            return Some(lyrics);
                        }
                    }
                    &"txt" => {
                        if let Some(lyrics) = Self::parse_txt_file(&lyric_path) {
                            return Some(lyrics);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        println!("未找到歌词文件: {}", audio_stem);
        None
    }

    /// 解析LRC格式歌词文件
    fn parse_lrc_file(lrc_path: &Path) -> Option<Vec<LyricLine>> {
        // 尝试多种编码方式读取文件
        let content = Self::read_file_with_encoding(lrc_path)?;
        
        let mut lyrics = Vec::new();
        
        for line_content in content.lines() {
            let line_content = line_content.trim();
            
            // 跳过空行和标签行（如[ar:], [ti:], [al:]等）
            if line_content.is_empty() || 
               (line_content.starts_with('[') && 
                (line_content.contains("ar:") || line_content.contains("ti:") || 
                 line_content.contains("al:") || line_content.contains("by:") ||
                 line_content.contains("offset:"))) {
                continue;
            }
            
            // 解析时间标签格式：[mm:ss.xx]歌词内容
            if let Some(lyric_line) = Self::parse_lrc_line(line_content) {
                lyrics.push(lyric_line);
            }
        }
        
        // 按时间排序
        lyrics.sort_by_key(|line| line.time);
        
        if lyrics.is_empty() {
            None
        } else {
            println!("成功解析歌词，共{}行", lyrics.len());
            Some(lyrics)
        }
    }

    /// 解析单行LRC歌词
    fn parse_lrc_line(line: &str) -> Option<LyricLine> {
        // 正则表达式匹配 [mm:ss.xx] 格式
        if !line.starts_with('[') {
            return None;
        }
        
        let end_bracket = line.find(']')?;
        let time_str = &line[1..end_bracket];
        let text = line[end_bracket + 1..].trim().to_string();
        
        // 解析时间 mm:ss.xx
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let minutes: u64 = parts[0].parse().ok()?;
        let seconds_parts: Vec<&str> = parts[1].split('.').collect();
        
        let seconds: u64 = seconds_parts[0].parse().ok()?;
        let milliseconds: u64 = if seconds_parts.len() > 1 {
            // 处理毫秒部分，确保是两位数
            let ms_str = seconds_parts[1];
            let ms_str = if ms_str.len() == 1 {
                format!("{}0", ms_str)
            } else if ms_str.len() > 2 {
                ms_str[..2].to_string()
            } else {
                ms_str.to_string()
            };
            ms_str.parse().unwrap_or(0) * 10 // 转换为毫秒
        } else {
            0
        };
        
        let total_milliseconds = minutes * 60 * 1000 + seconds * 1000 + milliseconds;
        
        Some(LyricLine {
            time: total_milliseconds,
            text,
        })
    }

    /// 解析普通文本格式歌词文件
    fn parse_txt_file(txt_path: &Path) -> Option<Vec<LyricLine>> {
        let content = Self::read_file_with_encoding(txt_path)?;
        
        let mut lyrics = Vec::new();
        let mut time_offset = 0u64;
        
        for line_content in content.lines() {
            let line_content = line_content.trim();
            
            if !line_content.is_empty() {
                lyrics.push(LyricLine {
                    time: time_offset,
                    text: line_content.to_string(),
                });
                
                // 每行间隔3秒（估算）
                time_offset += 3000;
            }
        }
        
        if lyrics.is_empty() {
            None
        } else {
            Some(lyrics)
        }
    }

    /// 使用多种编码方式读取文件内容
    fn read_file_with_encoding(file_path: &Path) -> Option<String> {
        // 首先尝试UTF-8编码
        if let Ok(content) = std::fs::read_to_string(file_path) {
            // 检查是否包含无效字符（乱码的迹象）
            if !content.contains('�') {
                println!("使用UTF-8编码成功读取歌词文件");
                return Some(content);
            }
        }
        
        // 如果UTF-8失败或包含乱码，尝试GBK编码
        if let Ok(bytes) = std::fs::read(file_path) {
            // 尝试使用encoding_rs库进行GBK解码
            let (decoded, _, had_errors) = encoding_rs::GBK.decode(&bytes);
            if !had_errors {
                println!("使用GBK编码成功读取歌词文件");
                return Some(decoded.into_owned());
            }
            
            // 如果GBK也失败，尝试GB2312
            let (decoded, _, had_errors) = encoding_rs::GB18030.decode(&bytes);
            if !had_errors {
                println!("使用GB18030编码成功读取歌词文件");
                return Some(decoded.into_owned());
            }
            
            // 最后尝试Windows-1252（西欧编码）
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
            println!("使用Windows-1252编码读取歌词文件（可能有问题）");
            return Some(decoded.into_owned());
        }
        
        println!("所有编码方式都失败，无法读取歌词文件");
        None
    }

    /// 策略1: 使用 lofty 库提取元数据和封面
    fn try_lofty_extraction(path: &Path) -> Option<SongInfo> {
        match Probe::open(path).and_then(|probe| probe.read()) {
            Ok(tagged_file) => {
                let path_str = path.to_string_lossy().into_owned();
                let tag = tagged_file.primary_tag()?;
                
                // 提取基本信息
                let title = tag.title().map(|s| s.to_string());
                let artist = tag.artist().map(|s| s.to_string());
                let album = tag.album().map(|s| s.to_string());
                
                // 提取封面
                let album_cover = Self::extract_cover_from_lofty(&tagged_file)
                    .or_else(|| Self::get_default_album_cover());
                
                // 提取时长
                let duration = tagged_file.properties().duration().as_secs();
                let duration = if duration > 0 && duration < 10800 { Some(duration) } else { None };
                
                println!("lofty 提取结果: title={:?}, artist={:?}, cover={}", 
                    title, artist, album_cover.is_some());
                
                Some(SongInfo {
                    path: path_str,
                    title,
                    artist,
                    album,
                    album_cover,
                    duration,
                    lyrics: None, // 默认没有歌词
                    media_type: Some(MediaType::Audio),
                    mv_path: None,
                    video_thumbnail: None,
                    has_lyrics: None,
                })
            }
            Err(e) => {
                println!("lofty 提取失败: {}", e);
                None
            }
        }
    }

    /// 策略2: 使用 audiotags 库提取元数据和封面  
    fn try_audiotags_extraction(path: &Path) -> Option<SongInfo> {
        match AudioTag::new().read_from_path(path) {
            Ok(tag) => {
                let path_str = path.to_string_lossy().into_owned();
                
                // 提取基本信息
                let title = tag.title().map(|s| s.to_string());
                let artist = tag.artist().map(|s| s.to_string());
                let album = tag.album_title().map(|s| s.to_string());
                
                // 提取封面
                let album_cover = if let Some(artwork) = tag.album_cover() {
                    match Self::convert_image_to_base64(&artwork.data) {
                        Ok(base64_string) => {
                            let mime_type = match artwork.mime_type {
                                audiotags::MimeType::Jpeg => "image/jpeg",
                                audiotags::MimeType::Png => "image/png",
                                _ => "image/jpeg",
                            };
                            let data_url = format!("data:{};base64,{}", mime_type, base64_string);
                            println!("从 audiotags 成功提取封面，MIME类型: {}", mime_type);
                            Some(data_url)
                        }
                        Err(e) => {
                            println!("audiotags 封面转换失败: {}", e);
                            None
                        }
                    }
                } else {
                    println!("audiotags 未找到封面");
                    None
                }.or_else(|| Self::get_default_album_cover());
                
                // 提取时长
                let duration = tag.duration().map(|d| d as u64);
                
                println!("audiotags 提取结果: title={:?}, artist={:?}, cover={}", 
                    title, artist, album_cover.is_some());
                
                Some(SongInfo {
                    path: path_str,
                    title,
                    artist,
                    album,
                    album_cover,
                    duration,
                    lyrics: None,
                    media_type: Some(MediaType::Audio),
                    mv_path: None,
                    video_thumbnail: None,
                    has_lyrics: None,
                })
            }
            Err(e) => {
                println!("audiotags 提取失败: {}", e);
                None
            }
        }
    }

    /// 策略3: 使用格式特定的方法（原有方法）
    fn try_format_specific_extraction(path: &Path) -> Option<SongInfo> {
        match Tag::read_from_path(path) {
            Ok(tag) => {
                // 提取专辑封面
                let album_cover = Self::extract_album_cover(&tag);
                
                // 尝试从ID3标签获取时长
                let duration = tag.duration().map(|d| d as u64);

                println!("格式特定方法提取结果: title={:?}, artist={:?}, cover={}", 
                    tag.title(), tag.artist(), album_cover.is_some());

                Some(SongInfo {
                    path: path.to_string_lossy().into_owned(),
                    title: tag.title().map(|s| s.to_string()),
                    artist: tag.artist().map(|s| s.to_string()),
                    album: tag.album().map(|s| s.to_string()),
                    album_cover,
                    duration,
                    lyrics: None,
                    media_type: Some(MediaType::Audio),
                    mv_path: None,
                    video_thumbnail: None,
                    has_lyrics: None,
                })
            }
            Err(e) => {
                println!("格式特定方法提取失败: {}", e);
                None
            }
        }
    }

    /// 策略4: 创建兜底歌曲信息
    fn create_fallback_song_info(path: &Path) -> SongInfo {
        let path_str = path.to_string_lossy().into_owned();
        
        // 尝试获取时长
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let duration = Self::get_accurate_duration(path, &ext);
        
        SongInfo {
            path: path_str,
            title: path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
            artist: None,
            album: None,
            album_cover: Self::get_default_album_cover(),
            duration,
            lyrics: None,
            media_type: Some(MediaType::Audio),
            mv_path: None,
            video_thumbnail: None,
            has_lyrics: None,
        }
    }

    /// 从 lofty 提取封面
    fn extract_cover_from_lofty(tagged_file: &lofty::TaggedFile) -> Option<String> {
        if let Some(tag) = tagged_file.primary_tag() {
            for picture in tag.pictures() {
                match Self::convert_image_to_base64(&picture.data()) {
                    Ok(base64_string) => {
                        let mime_type = picture.mime_type()
                            .map(|mt| mt.as_str())
                            .unwrap_or("image/jpeg");
                        let data_url = format!("data:{};base64,{}", mime_type, base64_string);
                        println!("从 lofty 成功提取封面，MIME类型: {}", mime_type);
                        return Some(data_url);
                    }
                    Err(e) => {
                        println!("lofty 封面转换失败: {}", e);
                        continue;
                    }
                }
            }
        }
        println!("lofty 未找到封面");
        None
    }

    /// 从ID3标签提取专辑封面
    fn extract_album_cover(tag: &Tag) -> Option<String> {
        let pictures: Vec<_> = tag.pictures().collect();

        if let Some(picture) = pictures.first() {
            match Self::convert_image_to_base64(&picture.data) {
                Ok(base64_string) => {
                    let mime_type = match picture.mime_type.as_str() {
                        "image/jpeg" => "image/jpeg",
                        "image/png" => "image/png",
                        _ => "image/jpeg",
                    };
                    let data_url = format!("data:{};base64,{}", mime_type, base64_string);
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else {
            Self::get_default_album_cover()
        }
    }

    /// 获取默认专辑封面
    fn get_default_album_cover() -> Option<String> {
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
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        }

        Self::generate_fallback_cover()
    }

    /// 生成一个简单的颜色块作为默认封面
    fn generate_fallback_cover() -> Option<String> {
        let mut img = RgbImage::new(300, 300);

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
    }

    /// 将图片数据转换为Base64字符串
    fn convert_image_to_base64(image_data: &[u8]) -> Result<String> {
        let img = image::load_from_memory(image_data)?;
        let resized_img = img.resize(300, 300, image::imageops::FilterType::Lanczos3);

        let mut jpeg_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_bytes);
        resized_img.write_to(&mut cursor, ImageFormat::Jpeg)?;

        let base64_string = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);
        Ok(base64_string)
    }

    /// 获取文件的准确时长（支持多种音频格式）
    fn get_accurate_duration(path: &Path, ext: &str) -> Option<u64> {
        println!("正在获取文件时长: {}", path.display());
        
        if let Some(duration) = Self::try_rodio_duration(path) {
            println!("通过rodio获取到时长: {}秒", duration);
            return Some(duration);
        }
        
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
        
        let estimated = Self::estimate_duration_from_filesize(path, ext);
        if let Some(d) = estimated {
            println!("通过文件大小估算时长: {}秒", d);
        }
        
        estimated
    }

    /// 尝试使用rodio解码器获取时长
    fn try_rodio_duration(path: &Path) -> Option<u64> {
        for attempt in 0..3 {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(source) = rodio::Decoder::new(reader) {
                    use rodio::Source;
                    if let Some(total_duration) = source.total_duration() {
                        let seconds = total_duration.as_secs();
                        if seconds > 0 && seconds < 10800 {
                            return Some(seconds);
                        }
                    }
                }
            }
            
            if attempt < 2 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        None
    }

    /// 简化的时长获取方法
    fn get_ogg_duration_advanced(path: &Path) -> Option<u64> {
        Self::estimate_duration_from_filesize(path, "ogg")
    }

    fn get_mp3_duration(path: &Path) -> Option<u64> {
        if let Ok(tag) = Tag::read_from_path(path) {
            if let Some(duration) = tag.duration() {
                return Some(duration as u64);
            }
        }
        Self::estimate_duration_from_filesize(path, "mp3")
    }

    fn get_flac_duration(path: &Path) -> Option<u64> {
        Self::estimate_duration_from_filesize(path, "flac")
    }

    fn get_wav_duration(path: &Path) -> Option<u64> {
        Self::estimate_duration_from_filesize(path, "wav")
    }

    fn get_aac_duration(path: &Path) -> Option<u64> {
        Self::estimate_duration_from_filesize(path, "m4a")
    }

    /// 基于文件大小估算时长
    fn estimate_duration_from_filesize(path: &Path, ext: &str) -> Option<u64> {
        let metadata = std::fs::metadata(path).ok()?;
        let file_size_bytes = metadata.len() as f64;
        
        let bitrate = match ext {
            "mp3" => 160000.0,
            "flac" => 850000.0,
            "wav" => 1411200.0,
            "ogg" => 112000.0,
            "m4a" | "aac" => 128000.0,
            "wma" => 128000.0,
            _ => 128000.0,
        };
        
        let estimated_seconds = (file_size_bytes / (bitrate / 8.0)).round() as u64;
        
        if estimated_seconds > 0 && estimated_seconds < 10800 {
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
#[derive(Debug)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    SetSong(usize),
    AddSong(SongInfo),
    AddSongs(Vec<SongInfo>),
    RemoveSong(usize),
    ClearPlaylist,
    SetPlayMode(PlayMode),
    SetVolume(f32),
    SeekTo(u64),
    UpdateVideoProgress { position: u64, duration: u64 },
}
