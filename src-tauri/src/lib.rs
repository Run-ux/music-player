mod player_fixed;
mod player_safe;

use crate::player_fixed::{PlayMode, PlayerCommand, PlayerEvent, PlayerState, SongInfo};
use crate::player_safe::SafePlayerManager as AudioPlayer;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State}; // Added Runtime, AppHandle, State, Manager, Emitter
use tauri_plugin_dialog::{DialogExt, FilePath}; // FilePath might be needed for the callback
use tokio::sync::mpsc; // Added import for mpsc
use tokio::sync::Mutex as AsyncMutex;

/// 状态管理器
struct MusicPlayerState {
    player: Arc<AudioPlayer>,
    event_receiver: mpsc::Receiver<PlayerEvent>,
}

/// Tauri 状态
#[derive(Default)]
struct AppState<R: Runtime> {
    player: Option<Arc<AsyncMutex<MusicPlayerState>>>,
    _marker: std::marker::PhantomData<R>, // Prevents AppState<R> from being object-safe
}

#[derive(serde::Serialize, Clone)]
struct InitialPlayerState {
    songs: Vec<SongInfo>,
    current_song_index: Option<usize>,
    is_playing: bool,
    volume: f32,
    play_mode: PlayMode,
    // No progress here, as it's hard to get accurately with current rodio
}

// 初始化播放器
/// 初始化播放器
#[tauri::command]
async fn init_player(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    // 获取状态锁
    let mut app_state = state.lock().await;

    // 如果播放器已经初始化，则不重复初始化
    if app_state.player.is_some() {
        // If already initialized, perhaps send current state to frontend?
        // Or simply return Ok(()) if the frontend is expected to call get_initial_player_state separately.
        return Ok(());
    }

    // 创建AudioPlayer实例（SafePlayerManager）和事件接收器
    let (audio_player, event_rx) = AudioPlayer::new();

    let player_state = MusicPlayerState {
        player: Arc::new(audio_player),
        event_receiver: event_rx,
    };

    let player_state = Arc::new(AsyncMutex::new(player_state));

    // 存储状态
    app_state.player = Some(player_state.clone());

    // 启动事件监听器
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        let mut player_state_guard = player_state.lock().await;
        while let Some(event) = player_state_guard.event_receiver.recv().await {
            match &event {
                PlayerEvent::Error(err) => println!("播放器错误: {}", err),
                _ => {}
            }
            // 将事件发送到前端
            let _ = app_handle_clone.emit_to("main", "player-event", event.clone());
            // Emit to main window
        }
    });

    Ok(())
}

// 获取播放器状态
/// 获取播放器状态
#[tauri::command]
async fn get_player_state(
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<PlayerState, String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_state())
}

// 获取播放列表
/// 获取播放列表
#[tauri::command]
async fn get_playlist(
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<Vec<SongInfo>, String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_playlist())
}

// 获取当前播放索引
/// 获取当前播放索引
#[tauri::command]
async fn get_current_index(
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<Option<usize>, String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_current_index())
}

// 获取播放模式
/// 获取播放模式
#[tauri::command]
async fn get_play_mode(
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<PlayMode, String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_play_mode())
}

// 播放
/// 播放
#[tauri::command]
async fn play(state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Play)
        .await
        .map_err(|e| e.to_string())
}

/// 暂停
#[tauri::command]
async fn pause(state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Pause)
        .await
        .map_err(|e| e.to_string())
}

/// 停止
#[tauri::command]
async fn stop(state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Stop)
        .await
        .map_err(|e| e.to_string())
}

/// 下一曲
#[tauri::command]
async fn next(state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Next)
        .await
        .map_err(|e| e.to_string())
}

/// 上一曲
#[tauri::command]
async fn previous(state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::Previous)
        .await
        .map_err(|e| e.to_string())
}

/// 设置当前歌曲
#[tauri::command]
async fn set_song(
    index: usize,
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SetSong(index))
        .await
        .map_err(|e| e.to_string())
}

/// 添加歌曲
#[tauri::command]
async fn add_song(
    path: String,
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::AddSong(PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())
}

/// 移除歌曲
#[tauri::command]
async fn remove_song(
    index: usize,
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::RemoveSong(index))
        .await
        .map_err(|e| e.to_string())
}

/// 清空播放列表
#[tauri::command]
async fn clear_playlist(
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::ClearPlaylist)
        .await
        .map_err(|e| e.to_string())
}

/// 设置播放模式
#[tauri::command]
async fn set_play_mode(
    mode: PlayMode,
    state: tauri::State<'_, AsyncMutex<AppState<tauri::Wry>>>,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let player_state = app_state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    player_state_guard
        .player
        .send_command(PlayerCommand::SetPlayMode(mode))
        .await
        .map_err(|e| e.to_string())
}

/// 打开文件对话框添加歌曲
#[tauri::command]
async fn open_audio_files<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, AppState<R>>, // Corrected: Removed unnecessary generic for AppState
) -> Result<(), String> {
    println!("Attempting to open audio files...");
    let player = state.player.clone();
    let app_handle_clone = app_handle.clone();

    // Spawn a tokio task to handle the async dialog and subsequent processing
    tokio::spawn(async move {
        // Use pick_files_async and await its result
        let picked_files_result = app_handle_clone
            .dialog()
            .file()
            .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
            .set_title("Select Audio Files")
            .pick_files_async()
            .await;

        match picked_files_result {
            Ok(Some(files)) => {
                // pick_files_async returns Result<Option<Vec<FilePath>>, Error>
                if !files.is_empty() {
                    println!("Files picked: {:?}", files);
                    let mut songs_to_add = Vec::new();
                    for file_path_type in files {
                        let path_buf: PathBuf = file_path_type.path; // Access the `path` field
                        println!("Processing file: {:?}", path_buf);

                        match SongInfo::from_path(&path_buf) {
                            Ok(song_info) => {
                                songs_to_add.push(song_info);
                            }
                            Err(e) => {
                                eprintln!("Error processing song from path {:?}: {}", path_buf, e);
                                if let Err(emit_err) = app_handle_clone.emit_to(
                                    "main",
                                    "player_error",
                                    &format!("Failed to load song {}: {}", path_buf.display(), e),
                                ) {
                                    eprintln!("Failed to emit player_error event: {:?}", emit_err);
                                }
                            }
                        }
                    }

                    if !songs_to_add.is_empty() {
                        if let Err(e) = player
                            .send_command(PlayerCommand::AddSongs(songs_to_add))
                            .await
                        {
                            eprintln!("Error sending AddSongs command: {:?}", e);
                            if let Err(emit_err) = app_handle_clone.emit_to(
                                "main",
                                "player_error",
                                &format!("Failed to add songs: {}", e),
                            ) {
                                eprintln!("Failed to emit player_error event: {:?}", emit_err);
                            }
                        } else {
                            println!("AddSongs command sent successfully.");
                            if let Err(emit_err) =
                                app_handle_clone.emit_to("main", "songs_added", &())
                            {
                                eprintln!("Failed to emit songs_added event: {:?}", emit_err);
                            }
                        }
                    }
                } else {
                    println!("No files selected.");
                }
            }
            Ok(None) => {
                println!("File dialog was cancelled.");
            }
            Err(e) => {
                eprintln!("Error picking files: {:?}", e);
                if let Err(emit_err) = app_handle_clone.emit_to(
                    "main",
                    "player_error",
                    &format!("Error opening file dialog: {}", e),
                ) {
                    eprintln!("Failed to emit player_error event: {:?}", emit_err);
                }
            }
        }
    });

    Ok(())
}

/// 获取初始播放器状态
#[tauri::command]
async fn get_initial_player_state<R: Runtime>(
    state: State<'_, AppState<R>>, // Corrected: AppState<R>
) -> Result<InitialPlayerState, String> {
    let snapshot = state.player.get_player_state_snapshot().await;
    Ok(InitialPlayerState {
        songs: snapshot.playlist,
        current_song_index: snapshot.current_index,
        is_playing: snapshot.state == PlayerState::Playing,
        volume: snapshot.volume,
        play_mode: snapshot.play_mode,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            open_audio_files,
            play_pause,
            stop_player,
            next_song,
            previous_song,
            set_song,
            remove_song_from_playlist,
            clear_player_playlist,
            set_player_play_mode,
            set_player_volume,
            seek_player_to,
            get_initial_player_state
        ])
        .run(tauri::generate_context!()) // Use generate_context!
        .expect("error while running tauri application");
}

// Define command handlers that were previously using the incorrect AppState

#[tauri::command]
async fn play_pause<R: Runtime>(state: State<'_, AppState<R>>) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::Play)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_player<R: Runtime>(state: State<'_, AppState<R>>) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::Stop)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn next_song<R: Runtime>(state: State<'_, AppState<R>>) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::Next)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn previous_song<R: Runtime>(state: State<'_, AppState<R>>) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::Previous)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_song<R: Runtime>(state: State<'_, AppState<R>>, index: usize) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::SetSong(index))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_song_from_playlist<R: Runtime>(
    state: State<'_, AppState<R>>,
    index: usize,
) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::RemoveSong(index))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_player_playlist<R: Runtime>(state: State<'_, AppState<R>>) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::ClearPlaylist)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_player_play_mode<R: Runtime>(
    state: State<'_, AppState<R>>,
    mode: PlayMode,
) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::SetPlayMode(mode))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_player_volume<R: Runtime>(
    state: State<'_, AppState<R>>,
    volume: f32,
) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::SetVolume(volume))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn seek_player_to<R: Runtime>(
    state: State<'_, AppState<R>>,
    position_secs: u64,
) -> Result<(), String> {
    state
        .player
        .send_command(PlayerCommand::SeekTo(position_secs))
        .await
        .map_err(|e| e.to_string())
}
