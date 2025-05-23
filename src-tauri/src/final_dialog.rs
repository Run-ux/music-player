/// 打开文件对话框添加歌曲 - 完全重写版本
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
    
    // 在单独的线程中启动文件选择
    let app_handle_clone = app_handle.clone();
    let player_state_clone = player_state.clone();
    
    // 启动单独的线程，因为文件对话框会阻塞当前线程
    std::thread::spawn(move || {
        // 使用同步API选择文件
        let file_paths = app_handle_clone
            .dialog()
            .file()
            .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
            .set_title("Select Audio Files")
            .allow_multiple(true)
            .pick();
            
        // 处理选择结果
        match file_paths {
            Some(paths) if !paths.is_empty() => {
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
            }
            _ => {
                println!("No files selected or dialog cancelled.");
            }
        }
    });
    
    // 立即返回，不等待文件选择
    Ok(())
}
