mod player_fixed;
mod player_safe;

use crate::player_fixed::{PlayMode, PlayerCommand, PlayerEvent, PlayerState, SongInfo};
use crate::player_safe::SafePlayerManager as AudioPlayer;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::mpsc; // Added import for mpsc
use tokio::sync::Mutex as AsyncMutex;

/// 状态管理器
struct MusicPlayerState {
    player: Arc<AudioPlayer>,
    event_receiver: mpsc::Receiver<PlayerEvent>,
}

/// Tauri 状态
#[derive(Default)]
struct AppState {
    player: Option<Arc<AsyncMutex<MusicPlayerState>>>,
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
async fn init_player<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 获取状态的唯一访问权
    let state_inner = state.inner();
    
    // 如果播放器已经初始化，则不重复初始化
    if state_inner.player.is_some() {
        // If already initialized, perhaps send current state to frontend?
        // Or simply return Ok(()) if the frontend is expected to call get_initial_player_state separately.
        return Ok(());
    }
    
    // 使用drop释放state_inner引用，解决引用可变性问题
    drop(state_inner);

    // 创建AudioPlayer实例（SafePlayerManager）和事件接收器
    let (audio_player, event_rx) = AudioPlayer::new();

    let player_state = MusicPlayerState {
        player: Arc::new(audio_player),
        event_receiver: event_rx,
    };

    let player_state = Arc::new(AsyncMutex::new(player_state));
    
    // 创建一个新的AppState实例并使用manage替换原有的实例
    let mut app_state = AppState::default();
    app_state.player = Some(player_state.clone());
    app_handle.manage::<AppState>(app_state);

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
        }
    });

    Ok(())
}

// 获取播放器状态
/// 获取播放器状态
#[tauri::command]
async fn get_player_state(
    state: tauri::State<'_, AppState>,
) -> Result<PlayerState, String> {
    let player_state = state
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
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SongInfo>, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_playlist())
}

/// 获取当前播放索引
#[tauri::command]
async fn get_current_index(
    state: tauri::State<'_, AppState>,
) -> Result<Option<usize>, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_current_index())
}

/// 获取播放模式
#[tauri::command]
async fn get_play_mode(
    state: tauri::State<'_, AppState>,
) -> Result<PlayMode, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_play_mode())
}

/// 播放
#[tauri::command]
async fn play(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
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
async fn pause(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
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
async fn stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
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
async fn next(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
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
async fn previous(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
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
    state: State<'_, AppState>, 
    index: usize
) -> Result<(), String> {
    let player_state = state
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
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    // 创建SongInfo对象代替直接使用PathBuf
    match SongInfo::from_path(&PathBuf::from(&path)) {
        Ok(song_info) => {
            player_state_guard
                .player
                .send_command(PlayerCommand::AddSong(song_info))
                .await
                .map_err(|e| e.to_string())
        }
        Err(e) => Err(format!("无法从路径创建歌曲信息: {}", e))
    }
}

/// 移除歌曲
#[tauri::command]
async fn remove_song(
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let player_state = state
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
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let player_state = state
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
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let player_state = state
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

/// 打开文件对话框添加歌曲 - 完全重写版本
#[tauri::command]
async fn open_audio_files<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Attempting to open audio files... (DEBUG LOG)");
    
    // 检查播放器状态
    println!("检查播放器状态：{:?}", state.player.is_some());
    
    // 如果播放器未初始化，尝试先初始化
    if state.player.is_none() {
        println!("播放器未初始化，尝试初始化...");
        // 创建AudioPlayer实例（SafePlayerManager）和事件接收器
        let (audio_player, event_rx) = AudioPlayer::new();

        let player_state = MusicPlayerState {
            player: Arc::new(audio_player),
            event_receiver: event_rx,
        };

        let player_state = Arc::new(AsyncMutex::new(player_state));
        
        // 创建一个新的AppState实例并使用manage替换原有的实例
        let mut app_state = AppState::default();
        app_state.player = Some(player_state.clone());
        app_handle.manage::<AppState>(app_state);
        
        println!("播放器初始化完成");
        
        // 获取新的状态
        let state = app_handle.state::<AppState>();
        println!("新的状态已获取，播放器是否存在：{:?}", state.player.is_some());
    }
    
    // 再次尝试获取播放器状态
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器仍然未初始化".to_string())?
        .clone();
    
    println!("播放器已初始化");

    // 在单独的线程中启动文件选择
    let app_handle_clone = app_handle.clone();
    let player_state_clone = player_state.clone();    // 启动单独的线程，因为文件对话框会阻塞当前线程
    std::thread::spawn(move || {
        println!("Starting file dialog in separate thread... (DEBUG LOG)");
        // 使用同步API选择文件，使用回调函数接收结果
        app_handle_clone
            .dialog()
            .file()
            .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
            .pick_files(move |file_paths| {
                if let Some(paths) = file_paths {
                    if paths.is_empty() {
                        println!("No files selected.");
                        return;
                    }
                    
                    println!("Selected {} files", paths.len());
                    let mut songs_to_add = Vec::new();
                    
                    for path in paths {
                        let path_str = path.to_string();
                        println!("Processing file: {}", path_str);
                        
                        // 创建标准路径
                        let path_buf = PathBuf::from(path_str);
                        
                        // 尝试加载歌曲信息
                        match SongInfo::from_path(&path_buf) {
                            Ok(song_info) => {
                                songs_to_add.push(song_info);
                            }
                            Err(e) => {
                                eprintln!("Error processing song: {}", e);
                            }
                        }
                    }
                    
                    // 如果有歌曲要添加
                    if !songs_to_add.is_empty() {
                        // 在当前线程中使用block_on处理异步操作
                        tauri::async_runtime::block_on(async {
                            let player_state_guard = player_state_clone.lock().await;
                            match player_state_guard
                                .player
                                .send_command(PlayerCommand::AddSongs(songs_to_add))
                                .await
                            {
                                Ok(_) => {
                                    println!("Songs added successfully!");
                                    // 发送事件通知前端
                                    let _ = app_handle_clone.emit_to("main", "songs_added", ());
                                }
                                Err(e) => {
                                    eprintln!("Error adding songs: {}", e);
                                    let _ = app_handle_clone.emit_to(
                                        "main", 
                                        "player_error",
                                        format!("Failed to add songs: {}", e)
                                    );
                                }
                            }
                        });
                    } else {
                        println!("No valid audio files were selected.");
                    }
                } else {
                    println!("Dialog was cancelled.");
                }
            });
    });
    
    // 立即返回，不等待文件选择
    Ok(())
}

/// 获取初始播放器状态
#[tauri::command]
async fn get_initial_player_state(
    state: State<'_, AppState>,
) -> Result<InitialPlayerState, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    
    // 使用默认音量1.0
    Ok(InitialPlayerState {
        songs: player_state_guard.player.get_playlist(),
        current_song_index: player_state_guard.player.get_current_index(),
        is_playing: player_state_guard.player.get_state() == PlayerState::Playing,
        volume: 1.0, // 使用默认音量值
        play_mode: player_state_guard.player.get_play_mode(),
    })
}

/// 应用程序设置函数
fn setup_app<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let app_state: AppState = AppState::default();
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
            stop,
            next,
            previous,
            set_song,
            add_song,
            remove_song,
            clear_playlist,
            set_play_mode,
            open_audio_files,
            get_initial_player_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
