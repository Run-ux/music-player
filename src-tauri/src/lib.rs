mod global_player;
mod player_fixed;
mod player_safe;

use crate::global_player::{GlobalPlayer, PlayerWrapper};
use crate::player_fixed::{PlayMode, PlayerCommand, PlayerEvent, PlayerState, SongInfo};
use crate::player_safe::SafePlayerManager as AudioPlayer;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex as AsyncMutex;

// 使用全局播放器模块中的PlayerWrapper

/// Tauri 应用状态
#[derive(Default)]
struct AppState {
    player: Option<Arc<AsyncMutex<PlayerWrapper>>>,
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
/// 初始化播放器 - 使用全局单例
#[tauri::command]
async fn init_player<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    state: tauri::State<'_, AppState>, // Note: this 'state' can be stale if init_player is called to fix it.
) -> Result<(), String> {
    println!("Init player called...");

    // 检查全局播放器是否已初始化
    if let Ok(global_player_lock) = GlobalPlayer::instance().lock() {
        if global_player_lock.is_initialized() {
            println!("全局播放器已经初始化，不需要重复初始化");

            // 确保Tauri状态中也有播放器引用
            // Fetch the current AppState directly from app_handle, as the 'state' argument might be stale.
            let current_tauri_app_state = app_handle.state::<AppState>();
            if current_tauri_app_state.player.is_none() {
                println!("Tauri AppState player is None, attempting to sync from GlobalPlayer.");
                if let Some(player_arc) = global_player_lock.get_player() {
                    println!("GlobalPlayer provided a player Arc. Updating AppState.");
                    let mut new_app_state = AppState::default();
                    new_app_state.player = Some(player_arc);
                    app_handle.manage::<AppState>(new_app_state); // Attempt to overwrite/set the state

                    // Verify immediately if the manage call was effective
                    let updated_state_check = app_handle.state::<AppState>();
                    if updated_state_check.player.is_some() {
                        println!("从全局播放器成功更新了Tauri状态，播放器现在存在于AppState中。");
                    } else {
                        println!("警告: 从全局播放器更新Tauri状态后，AppState中的播放器仍然为None。app_handle.manage() 可能未按预期生效。");
                    }
                } else {
                    println!("警告: 全局播放器已初始化，但其 get_player() 方法返回 None。无法使用播放器更新Tauri AppState。");
                }
            } else {
                println!("Tauri AppState player already exists. No update needed from GlobalPlayer in this path.");
            }
            return Ok(());
        }
    } else {
        return Err("无法获取全局播放器锁 (在检查初始化路径时)".to_string());
    }

    // 初始化全局播放器 (This path is for first-time initialization)
    println!("初始化全局播放器...");
    let (player_state_arc, mut event_rx) = match GlobalPlayer::instance().lock() {
        Ok(mut global_player) => global_player.initialize(),
        Err(_) => return Err("无法获取全局播放器锁 (在初始化播放器时)".to_string()),
    };

    println!("创建并设置应用状态 (首次初始化)...");
    let mut app_state_to_manage = AppState::default();
    app_state_to_manage.player = Some(player_state_arc.clone());
    app_handle.manage::<AppState>(app_state_to_manage); // Attempt to set the initial AppState with the player

    // 验证初始化成功
    let verify_state = app_handle.state::<AppState>();
    println!(
        "验证播放器已初始化 (首次调用后检查AppState): {}",
        verify_state.player.is_some()
    );
    if !verify_state.player.is_some() {
        println!("警告: 首次初始化播放器并尝试设置AppState后，通过 app_handle.state() 获取的AppState中播放器仍为None。app_handle.manage() 可能未按预期生效。");
    }

    // 启动事件监听器
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        println!("启动事件监听任务...");
        println!("事件监听器已准备好接收事件");
        while let Some(event) = event_rx.recv().await {
            match &event {
                PlayerEvent::Error(err) => println!("播放器错误: {}", err),
                PlayerEvent::StateChanged(state) => println!("播放状态变更: {:?}", state),
                PlayerEvent::SongChanged(idx, _) => println!("当前歌曲变更: {}", idx),
                _ => {}
            }
            // 将事件发送到前端
            if let Err(e) = app_handle_clone.emit_to("main", "player-event", event.clone()) {
                println!("无法发送事件到前端: {:?}", e);
            }
        }
        println!("事件监听任务结束");
    });

    println!("播放器初始化完成");
    Ok(())
}

// 获取播放器状态
/// 获取播放器状态
#[tauri::command]
async fn get_player_state(state: tauri::State<'_, AppState>) -> Result<PlayerState, String> {
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
async fn get_playlist(state: tauri::State<'_, AppState>) -> Result<Vec<SongInfo>, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_playlist())
}

/// 获取当前播放索引
#[tauri::command]
async fn get_current_index(state: tauri::State<'_, AppState>) -> Result<Option<usize>, String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
    Ok(player_state_guard.player.get_current_index())
}

/// 获取播放模式
#[tauri::command]
async fn get_play_mode(state: tauri::State<'_, AppState>) -> Result<PlayMode, String> {
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
async fn set_song(state: State<'_, AppState>, index: usize) -> Result<(), String> {
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
async fn add_song(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?;
    let player_state_guard = player_state.lock().await;
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
async fn remove_song(index: usize, state: tauri::State<'_, AppState>) -> Result<(), String> {
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
async fn clear_playlist(state: tauri::State<'_, AppState>) -> Result<(), String> {
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
async fn set_play_mode(mode: PlayMode, state: tauri::State<'_, AppState>) -> Result<(), String> {
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

/// 打开文件对话框添加歌曲 - 使用全局播放器
#[tauri::command]
async fn open_audio_files<R: Runtime>(
    app_handle: AppHandle<R>,
    // state: State<'_, AppState>, // Removed: will fetch fresh state via app_handle
) -> Result<(), String> {
    println!("尝试打开音频文件... (DEBUG LOG)");

    let mut player_arc_option: Option<Arc<AsyncMutex<PlayerWrapper>>> = None;

    // 1. Attempt to get player from AppState as a first check
    let current_app_state = app_handle.state::<AppState>();
    if current_app_state.player.is_some() {
        println!("从 AppState 获取到播放器 (初次检查)。");
        player_arc_option = current_app_state.player.clone();
    } else {
        println!("AppState 中没有播放器 (初次检查)。");
    }

    // 2. If not in AppState, proceed to check GlobalPlayer and potentially initialize
    if player_arc_option.is_none() {
        println!("AppState 无播放器，尝试从 GlobalPlayer 获取或初始化。");

        let mut needs_init_call = false;
        {
            // Scope for the first GlobalPlayer lock
            let global_player_guard = GlobalPlayer::instance()
                .lock()
                .map_err(|_| "无法锁定 GlobalPlayer (初次检查)".to_string())?;

            if global_player_guard.is_initialized() {
                println!("GlobalPlayer 已初始化，尝试获取其实例。");
                if let Some(p_global) = global_player_guard.get_player() {
                    player_arc_option = Some(p_global.clone());
                    println!("从已初始化的 GlobalPlayer 获取到播放器实例。");
                    // Best effort to update AppState
                    let mut temp_app_state = AppState {
                        player: Some(p_global.clone()),
                    };
                    app_handle.manage(temp_app_state);
                    let check_state = app_handle.state::<AppState>();
                    println!(
                        "尝试用 GlobalPlayer 实例更新 AppState 后，AppState.player.is_some(): {}",
                        check_state.player.is_some()
                    );
                } else {
                    println!("GlobalPlayer 已初始化但 get_player() 返回 None。");
                }
                // If player_arc_option is still None here, it means get_player() failed or AppState update didn't stick for some reason.
                // We might still need to call init_player if GlobalPlayer reported initialized but couldn't provide a player.
                // However, init_player typically handles the creation. If it's initialized but no player, that's a deeper issue.
                // For now, if GlobalPlayer says it's initialized, we trust that and don't call init_player again based on this lock.
                // needs_init_call will only be true if GlobalPlayer itself was not initialized.
            } else {
                println!("GlobalPlayer 未初始化。");
                needs_init_call = true;
            }
            // global_player_guard is dropped here as it goes out of scope
        }

        // If GlobalPlayer was not initialized (needs_init_call is true)
        // OR if it was initialized but we still didn't get a player_arc_option (e.g. get_player() returned None)
        // then we proceed to call init_player.
        if needs_init_call || player_arc_option.is_none() {
            if needs_init_call {
                println!("GlobalPlayer 未初始化，调用 init_player 命令。");
            } else {
                println!("GlobalPlayer 已初始化但未能获取播放器实例，尝试调用 init_player 以确保状态同步。");
            }

            let state_for_init_call = app_handle.state::<AppState>(); // Freshly get it
            init_player(app_handle.clone(), state_for_init_call).await?; // .await is now safe
            println!("init_player 调用完毕。再次尝试从 GlobalPlayer 获取。");

            {
                // Scope for the second GlobalPlayer lock, after init_player has run
                let global_player_guard_after_init = GlobalPlayer::instance()
                    .lock()
                    .map_err(|_| "无法锁定 GlobalPlayer (init_player 后)".to_string())?;

                if global_player_guard_after_init.is_initialized() {
                    if let Some(p_global_after_init) = global_player_guard_after_init.get_player() {
                        player_arc_option = Some(p_global_after_init.clone());
                        println!(
                            "从 GlobalPlayer 获取到播放器 (init_player 后)。再次尝试更新 AppState。"
                        );
                        let mut temp_app_state_after_init = AppState {
                            player: Some(p_global_after_init.clone()),
                        };
                        app_handle.manage(temp_app_state_after_init);
                        let check_state_after_init = app_handle.state::<AppState>();
                        println!(
                            "再次尝试更新 AppState (init_player 后)，AppState.player.is_some(): {}",
                            check_state_after_init.player.is_some()
                        );
                    } else {
                        println!(
                            "GlobalPlayer 初始化后 (经 init_player)，get_player() 仍返回 None。"
                        );
                    }
                } else {
                    println!("init_player 调用后，GlobalPlayer 仍未初始化。");
                }
                // global_player_guard_after_init is dropped here
            }
        }
    }

    let player_instance_for_dialog =
        player_arc_option.ok_or_else(|| "最终未能获取到播放器实例以打开文件对话框".to_string())?;

    println!("播放器已成功获取/初始化，准备打开文件选择对话框");

    let app_handle_clone = app_handle.clone();
    let player_arc_clone_for_thread = player_instance_for_dialog.clone();

    std::thread::spawn(move || {
        println!("Opening file dialog in new thread... (DEBUG LOG)");
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
                        let path_buf = PathBuf::from(&path_str);
                        println!("Processing file: {}", path_str);
                        match SongInfo::from_path(&path_buf) {
                            Ok(song_info) => {
                                songs_to_add.push(song_info);
                            }
                            Err(e) => {
                                eprintln!("Error processing song {}: {}", path_str, e);
                                let _ = app_handle_clone.emit_to(
                                    "main",
                                    "player_error",
                                    format!("无法处理歌曲 {}: {}", path_str, e),
                                );
                            }
                        }
                    }

                    if !songs_to_add.is_empty() {
                        tauri::async_runtime::block_on(async {
                            let player_wrapper_guard = player_arc_clone_for_thread.lock().await;
                            match player_wrapper_guard
                                .player // Assuming PlayerWrapper has a public 'player' field of type AudioPlayer
                                .send_command(PlayerCommand::AddSongs(songs_to_add))
                                .await
                            {
                                Ok(_) => {
                                    println!("Songs added successfully!");
                                    let _ = app_handle_clone.emit_to("main", "songs_added", ());
                                }
                                Err(e) => {
                                    eprintln!("Error adding songs: {}", e);
                                    let _ = app_handle_clone.emit_to(
                                        "main",
                                        "player_error",
                                        format!("Failed to add songs: {}", e),
                                    );
                                }
                            }
                        });
                    } else {
                        println!("No valid audio files were selected or processed.");
                    }
                } else {
                    println!("Dialog was cancelled.");
                }
            });
    });
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

/// 应用程序设置函数 - 使用全局播放器
fn setup_app<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    println!("设置应用程序...");

    // 检查全局播放器是否已初始化
    let player_opt = match GlobalPlayer::instance().lock() {
        Ok(global_player) => {
            if global_player.is_initialized() {
                println!("全局播放器已初始化，使用现有播放器");
                global_player.get_player()
            } else {
                println!("全局播放器未初始化，暂不创建");
                None
            }
        }
        Err(_) => {
            println!("无法获取全局播放器锁，使用空状态");
            None
        }
    };

    // 创建AppState实例
    let app_state = AppState { player: player_opt };
    app.manage(app_state);
    println!("应用程序设置完成");
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
