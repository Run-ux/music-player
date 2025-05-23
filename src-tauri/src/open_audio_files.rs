/// 打开文件对话框添加歌曲 - 使用Tauri 2.0 Dialog API
#[tauri::command]
async fn open_audio_files<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Attempting to open audio files...");
    
    // 先检查player是否初始化
    let player_state = state
        .player
        .as_ref()
        .ok_or_else(|| "播放器未初始化".to_string())?
        .clone();
    
    // 创建克隆用于回调
    let app_handle_clone = app_handle.clone();
    let player_state_clone = player_state.clone();
    
    // 使用Tauri dialog API
    app_handle
        .dialog()
        .file()
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
        .set_title("Select Audio Files")
        .pick_files(move |files_opt| {
            if let Some(files) = files_opt {
                if !files.is_empty() {
                    println!("Files picked: {:?}", files);
                    let mut songs_to_add = Vec::new();
                    
                    for file_path in files {
                        // 对于Tauri 2, FilePath需要转换为Path
                        match file_path.try_into_path() {
                            Ok(actual_path) => {
                                println!("Processing file: {:?}", actual_path);
                                match SongInfo::from_path(&actual_path) {
                                    Ok(song_info) => {
                                        songs_to_add.push(song_info);
                                    }
                                    Err(e) => {
                                        eprintln!("Error processing song from path {:?}: {}", actual_path, e);
                                        let _ = app_handle_clone.emit_to(
                                            "main",
                                            "player_error",
                                            &format!("Failed to load song {}: {}", actual_path.display(), e),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error converting file path: {}", e);
                                let _ = app_handle_clone.emit_to(
                                    "main",
                                    "player_error",
                                    &format!("Failed to process file: {}", e),
                                );
                            }
                        }
                    }
                    
                    // 如果有歌曲添加到列表，发送命令
                    if !songs_to_add.is_empty() {
                        // 使用tauri的异步运行时
                        tauri::async_runtime::spawn(async move {
                            let player_state_guard = player_state_clone.lock().await;
                            if let Err(e) = player_state_guard
                                .player
                                .send_command(PlayerCommand::AddSongs(songs_to_add))
                                .await
                            {
                                eprintln!("Error sending AddSongs command: {:?}", e);
                                let _ = app_handle_clone.emit_to(
                                    "main",
                                    "player_error",
                                    &format!("Failed to add songs: {}", e),
                                );
                            } else {
                                println!("AddSongs command sent successfully.");
                                let _ = app_handle_clone.emit_to("main", "songs_added", &());
                            }
                        });
                    } else {
                        println!("No valid songs to add.");
                    }
                } else {
                    println!("No files selected.");
                }
            } else {
                println!("File dialog was cancelled.");
            }
        });
    
    // 立即返回，因为实际的文件处理将在回调中进行
    Ok(())
}
