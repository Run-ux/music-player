mod global_player;
mod player_fixed;
mod player_safe;

use crate::global_player::{GlobalPlayer, PlayerWrapper};
use crate::player_fixed::{PlayMode, PlayerCommand, PlayerEvent, PlayerState, SongInfo};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex as AsyncMutex;

/// Tauri 应用状态 - 现在使用 GlobalPlayer 单例，不再需要存储播放器实例
#[derive(Default, Clone)]
struct AppState {
    // 保留结构以便将来扩展
}

/// 获取播放器实例的辅助函数
async fn get_player_instance() -> Result<Arc<AsyncMutex<PlayerWrapper>>, String> {
    let global_player_guard = GlobalPlayer::instance()
        .lock()
        .map_err(|_| "无法锁定 GlobalPlayer".to_string())?;

    global_player_guard
        .get_player()
        .ok_or_else(|| "播放器未初始化".to_string())
}

#[derive(serde::Serialize, Clone)]
struct InitialPlayerState {
    songs: Vec<SongInfo>,
    current_song_index: Option<usize>,
    is_playing: bool,
    volume: f32,
    play_mode: PlayMode,
}

/// 初始化播放器 - 简化版本
#[tauri::command]
async fn init_player<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 检查 GlobalPlayer 是否已经初始化
    {
        let global_player_guard = GlobalPlayer::instance()
            .lock()
            .map_err(|_| "无法获取全局播放器锁".to_string())?;

        if global_player_guard.is_initialized() {
            return Ok(());
        }
    }

    // 初始化全局播放器
    let (_player_state_arc, mut event_rx) = match GlobalPlayer::instance().lock() {
        Ok(mut global_player) => global_player.initialize(),
        Err(_) => return Err("无法获取全局播放器锁进行初始化".to_string()),
    };

    // 启动事件监听器
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // 记录错误事件
            if let PlayerEvent::Error(err) = &event {
                eprintln!("播放器错误: {}", err);
            }

            // 发送事件到前端
            if let Err(e) = app_handle_clone.emit("player-event", event.clone()) {
                eprintln!("发送事件到前端失败: {:?}", e);
            }
        }
    });

    Ok(())
}

/// 获取播放器状态
#[tauri::command]
async fn get_player_state(_state: tauri::State<'_, AppState>) -> Result<PlayerState, String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    Ok(player_state_guard.player.get_state())
}

/// 获取播放列表
#[tauri::command]
async fn get_playlist(_state: tauri::State<'_, AppState>) -> Result<Vec<SongInfo>, String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    Ok(player_state_guard.player.get_playlist())
}

/// 获取当前播放索引
#[tauri::command]
async fn get_current_index(_state: tauri::State<'_, AppState>) -> Result<Option<usize>, String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    Ok(player_state_guard.player.get_current_index())
}

/// 获取播放模式
#[tauri::command]
async fn get_play_mode(_state: tauri::State<'_, AppState>) -> Result<PlayMode, String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    Ok(player_state_guard.player.get_play_mode())
}

/// 播放
#[tauri::command]
async fn play(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Play)
        .await
        .map_err(|e| e.to_string())
}

/// 暂停
#[tauri::command]
async fn pause(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Pause)
        .await
        .map_err(|e| e.to_string())
}

/// 下一曲
#[tauri::command]
async fn next(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Next)
        .await
        .map_err(|e| e.to_string())
}

/// 上一曲
#[tauri::command]
async fn previous(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Previous)
        .await
        .map_err(|e| e.to_string())
}

/// 设置当前歌曲
#[tauri::command]
async fn set_song(_state: State<'_, AppState>, index: usize) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SetSong(index))
        .await
        .map_err(|e| e.to_string())
}

/// 添加歌曲
#[tauri::command]
async fn add_song(path: String, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    // 创建SongInfo对象代替直接使用PathBuf
    match SongInfo::from_path(&PathBuf::from(&path)) {
        Ok(song_info) => player_state_guard
            .player
            .send_command(PlayerCommand::AddSong(song_info))
            .await
            .map_err(|e| e.to_string()),
        Err(e) => Err(format!("无法从路径创建歌曲信息: {}", e)),
    }
}

/// 移除歌曲
#[tauri::command]
async fn remove_song(index: usize, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::RemoveSong(index))
        .await
        .map_err(|e| e.to_string())
}

/// 清空播放列表
#[tauri::command]
async fn clear_playlist(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::ClearPlaylist)
        .await
        .map_err(|e| e.to_string())
}

/// 设置播放模式
#[tauri::command]
async fn set_play_mode(mode: PlayMode, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SetPlayMode(mode))
        .await
        .map_err(|e| e.to_string())
}

/// 跳转到指定位置
#[tauri::command]
async fn seek_to(position: u64, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SeekTo(position))
        .await
        .map_err(|e| e.to_string())
}

/// 打开文件对话框添加歌曲 - 扩展为支持音频和视频文件
#[tauri::command]
async fn open_audio_files<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 检查 GlobalPlayer 是否初始化，如果没有就初始化
    let is_initialized = {
        let global_player_guard = GlobalPlayer::instance()
            .lock()
            .map_err(|_| "无法锁定 GlobalPlayer".to_string())?;
        global_player_guard.is_initialized()
    };

    if !is_initialized {
        init_player(app_handle.clone(), state).await?;
    }

    // 获取播放器实例
    let player_instance = {
        let global_player_guard = GlobalPlayer::instance()
            .lock()
            .map_err(|_| "无法锁定 GlobalPlayer".to_string())?;

        global_player_guard
            .get_player()
            .ok_or_else(|| "无法获取播放器实例".to_string())?
    };

    // 启动新线程处理文件对话框
    let app_handle_clone = app_handle.clone();
    let player_clone = player_instance.clone();

    std::thread::spawn(move || {
        app_handle_clone
            .dialog()
            .file()
            .add_filter("音频文件", &["mp3", "wav", "ogg", "flac", "m4a", "aac"])
            .add_filter("视频文件", &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"])
            .add_filter("所有媒体文件", &["mp3", "wav", "ogg", "flac", "m4a", "aac", "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"])
            .set_title("选择音频或视频文件")
            .pick_files(move |file_paths| {
                if let Some(paths) = file_paths {
                    if paths.is_empty() {
                        return;
                    }

                    let mut songs_to_add = Vec::new(); // 处理每个选中的文件
                    for path in paths {
                        let path_str = path.to_string();

                        match SongInfo::from_path(&PathBuf::from(&path_str)) {
                            Ok(song_info) => {
                                songs_to_add.push(song_info);
                            }
                            Err(e) => {
                                eprintln!("处理媒体文件失败 {}: {}", path_str, e);
                            }
                        }
                    } // 如果有有效的媒体文件，添加到播放器
                    if !songs_to_add.is_empty() {
                        tauri::async_runtime::block_on(async {
                            let player_guard = player_clone.lock().await;
                            match player_guard
                                .player
                                .send_command(PlayerCommand::AddSongs(songs_to_add))
                                .await
                            {
                                Ok(_) => {
                                    // 发送songs_added事件
                                    let _ = app_handle_clone.emit("songs_added", ());

                                    // 同时手动触发播放列表更新，确保前端能收到
                                    // 获取最新的播放列表
                                    let updated_playlist = player_guard.player.get_playlist();
                                    let _ = app_handle_clone.emit(
                                        "player-event",
                                        crate::player_fixed::PlayerEvent::PlaylistUpdated(
                                            updated_playlist,
                                        ),
                                    );
                                }
                                Err(e) => {
                                    eprintln!("添加媒体文件失败: {}", e);
                                    let _ = app_handle_clone
                                        .emit("player_error", format!("添加媒体文件失败: {}", e));
                                }
                            }
                        });
                    }
                }
            });
    });
    Ok(())
}

/// 获取视频流数据 - 用于前端播放视频
#[tauri::command]
async fn get_video_stream(file_path: String) -> Result<Vec<u8>, String> {
    println!("开始读取视频文件: {}", file_path);
    
    // 检查文件是否存在
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("视频文件不存在: {}", file_path));
    }
    
    // 读取视频文件
    match std::fs::read(&file_path) {
        Ok(data) => {
            println!("成功读取视频文件，大小: {} 字节", data.len());
            Ok(data)
        }
        Err(e) => {
            eprintln!("读取视频文件失败: {}", e);
            Err(format!("读取视频文件失败: {}", e))
        }
    }
}

#[tauri::command]
async fn get_initial_player_state(
    _state: State<'_, AppState>,
) -> Result<InitialPlayerState, String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;

    // 使用默认音量1.0
    Ok(InitialPlayerState {
        songs: player_state_guard.player.get_playlist(),
        current_song_index: player_state_guard.player.get_current_index(),
        is_playing: player_state_guard.player.get_state() == PlayerState::Playing,
        volume: 1.0, // 使用默认音量值
        play_mode: player_state_guard.player.get_play_mode(),
    })
}

/// 应用程序设置函数 - 简化版本
fn setup_app<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个空的 AppState
    let app_state = AppState {};
    app.manage(app_state);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            init_player,
            get_player_state,
            get_playlist,
            get_current_index,
            get_play_mode,
            play,
            pause,
            next,
            previous,
            set_song,
            add_song,
            remove_song,
            clear_playlist,
            set_play_mode,
            seek_to,
            open_audio_files,
            get_initial_player_state,
            get_video_stream,
            update_video_progress,
            toggle_playback_mode,
            set_playback_mode,
            get_current_playback_mode, // 添加获取当前播放模式的命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 更新视频播放进度 - 专门用于视频文件的进度同步
#[tauri::command]
async fn update_video_progress(position: u64, duration: u64, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    
    // 直接发送进度更新事件，不通过播放器命令队列
    if let Some(current_idx) = player_state_guard.player.get_current_index() {
        let playlist = player_state_guard.player.get_playlist();
        if let Some(song) = playlist.get(current_idx) {
            // 只有当前播放的是视频文件时才处理
            if song.media_type == Some(crate::player_fixed::MediaType::Video) {
                // 这里我们需要直接访问事件发送器来发送进度更新
                // 由于架构限制，我们通过特殊的命令来处理视频进度
                player_state_guard
                    .player
                    .send_command(PlayerCommand::UpdateVideoProgress { position, duration })
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    
    Ok(())
}

/// 切换播放模式（音频/MV）
#[tauri::command]
async fn toggle_playback_mode(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::TogglePlaybackMode)
        .await
        .map_err(|e| e.to_string())
}

/// 设置播放模式（音频/MV）
#[tauri::command]
async fn set_playback_mode(mode: crate::player_fixed::MediaType, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_instance = get_player_instance().await?;
    let player_state_guard = player_instance.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SetPlaybackMode(mode))
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前播放模式
#[tauri::command]
async fn get_current_playback_mode(_state: tauri::State<'_, AppState>) -> Result<crate::player_fixed::MediaType, String> {
    let player_instance = get_player_instance().await?;
    let _player_state_guard = player_instance.lock().await;
    // 这里需要从播放器状态中获取当前播放模式
    // 目前先返回默认的Audio模式，稍后会修复
    Ok(crate::player_fixed::MediaType::Audio)
}
