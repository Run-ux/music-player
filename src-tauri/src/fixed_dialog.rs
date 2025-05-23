/// 打开文件对话框添加歌曲 - 简化版本
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
    
    // 使用JS回调方式，不再使用Rust的回调函数
    let player_state_clone = player_state.clone();
    let app_handle_clone = app_handle.clone();
    
    // 启动一个新线程处理文件选择
    std::thread::spawn(move || {
        // 使用dialog的同步API
        let file_paths = app_handle_clone
            .dialog()
            .file()
            .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
            .set_title("Select Audio Files")
            .allow_multiple(true)
            .pick();
        
        // 处理文件选择结果
        if let Some(paths) = file_paths {
            if paths.is_empty() {
                println!("No files selected.");
                return;
            }
            
            println!("Files picked: {:?}", paths);
            // 添加更详细的日志
            for (i, path) in paths.iter().enumerate() {
                println!("File {}: {:?}", i, path);
            }
            let mut songs_to_add = Vec::new();
            
            // 处理每个文件
            for path in paths {
                // FilePath 到 std::path::PathBuf 的转换
                let path_str = path.to_string();
                println!("Path string: {}", path_str);
                
                // 创建PathBuf
                let path_buf = PathBuf::from(path_str);
                println!("Processing path: {:?}", path_buf);
                
                match SongInfo::from_path(&path_buf) {
                    Ok(song_info) => {
                        songs_to_add.push(song_info);
                    }
                    Err(e) => {
                        eprintln!("Error processing song: {}", e);
                        // 简化错误处理
                    }
                }
            }
            
            // 如果有歌曲要添加
            if !songs_to_add.is_empty() {
                // 使用Tauri的异步运行时来处理异步操作
                tauri::async_runtime::block_on(async {
                    let player_state_guard = player_state_clone.lock().await;
                    if let Err(e) = player_state_guard
                        .player
                        .send_command(PlayerCommand::AddSongs(songs_to_add))
                        .await
                    {
                        eprintln!("Error adding songs: {}", e);
                        let _ = app_handle_clone.emit_to(
                            "main", 
                            "player_error",
                            format!("Failed to add songs: {}", e)
                        );
                    } else {
                        println!("Songs added successfully!");
                        // 通知前端更新UI
                        let _ = app_handle_clone.emit_to("main", "songs_added", ());
                    }
                });
            }
        } else {
            println!("Dialog was cancelled.");
        }
    });
    
    // 立即返回，不等待文件选择
    Ok(())
}
