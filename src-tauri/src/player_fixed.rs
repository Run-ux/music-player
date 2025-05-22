use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Result;
use id3::{Tag, TagLike};
use serde::{Deserialize, Serialize};
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

impl SongInfo {
    /// 从文件路径创建歌曲信息
    pub fn from_path(path: &Path) -> Result<Self> {
        let path_str = path.to_string_lossy().into_owned();
        let mut song_info = match Tag::read_from_path(path) {
            Ok(tag) => SongInfo {
                path: path_str,
                title: tag.title().map(|s| s.to_string()),
                artist: tag.artist().map(|s| s.to_string()),
                album: tag.album().map(|s| s.to_string()),
                album_cover: None, // 专辑封面暂时不处理
                duration: None,
            },
            Err(_) => {
                // 如果无法读取标签，则使用文件名作为标题
                SongInfo {
                    path: path_str,
                    title: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                    artist: None,
                    album: None,
                    album_cover: None,
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
    }
}

/// 播放器事件
#[derive(Debug, Clone, Serialize)]
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
    AddSong(SongInfo), // Changed from PathBuf to SongInfo
    AddSongs(Vec<SongInfo>), // Added new variant for adding multiple songs
    RemoveSong(usize),
    ClearPlaylist,
    SetPlayMode(PlayMode),
    SetVolume(f32), // Added SetVolume command
    SeekTo(u64),    // Added SeekTo command (position in seconds)
}
