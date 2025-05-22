use crate::player_fixed::{PlayMode, PlayerCommand, PlayerEvent, PlayerState, SongInfo};
use rand::Rng; // Added for shuffle mode
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 线程安全的播放器适配器
/// 将处理分为两部分：前端可以访问的线程安全状态和后台播放器线程
pub struct SafePlayerState {
    state: PlayerState,
    playlist: Vec<SongInfo>,
    current_index: Option<usize>,
    play_mode: PlayMode,
    volume: f32, // Added volume field
}

impl Default for SafePlayerState {
    fn default() -> Self {
        Self {
            state: PlayerState::Stopped,
            playlist: Vec::new(),
            current_index: None,
            play_mode: PlayMode::Sequential,
            volume: 1.0, // Default volume
        }
    }
}

/// 音频播放器管理器
/// 处理与前端的交互，维护线程安全的状态
pub struct SafePlayerManager {
    state: Arc<Mutex<SafePlayerState>>,
    command_sender: mpsc::Sender<PlayerCommand>,
}

impl SafePlayerManager {
    /// 创建新的播放器管理器
    pub fn new() -> (Self, mpsc::Receiver<PlayerEvent>) {
        let (event_tx, event_rx) = mpsc::channel::<PlayerEvent>(100);
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlayerCommand>(100);

        // 创建线程安全状态
        let state = Arc::new(Mutex::new(SafePlayerState::default()));

        // 启动处理播放器命令的线程
        let state_clone = state.clone();
        let event_tx_clone = event_tx.clone();
        let cmd_tx_clone_for_thread = cmd_tx.clone(); // Clone sender for the thread

        std::thread::spawn(move || {
            if let Err(e) = run_player_thread(cmd_rx, event_tx_clone, state_clone, cmd_tx_clone_for_thread) {
                eprintln!("播放器线程错误: {}", e);
            }
        });

        (
            SafePlayerManager {
                state,
                command_sender: cmd_tx,
            },
            event_rx,
        )
    }

    /// 获取播放器状态
    pub fn get_state(&self) -> PlayerState {
        self.state.lock().unwrap().state
    }

    /// 获取当前播放列表
    pub fn get_playlist(&self) -> Vec<SongInfo> {
        self.state.lock().unwrap().playlist.clone()
    }

    /// 获取当前播放的歌曲索引
    pub fn get_current_index(&self) -> Option<usize> {
        self.state.lock().unwrap().current_index
    }

    /// 获取当前播放模式
    pub fn get_play_mode(&self) -> PlayMode {
        self.state.lock().unwrap().play_mode
    }

    // 获取播放器状态快照，用于初始化前端状态
    pub async fn get_player_state_snapshot(&self) -> SafePlayerStateSnapshot {
        let guard = self.state.lock().unwrap();
        SafePlayerStateSnapshot {
            state: guard.state,
            playlist: guard.playlist.clone(),
            current_index: guard.current_index,
            play_mode: guard.play_mode,
            volume: guard.volume, // Include volume
        }
    }

    /// 发送命令到播放器
    pub async fn send_command(&self, cmd: PlayerCommand) -> Result<(), anyhow::Error> {
        self.command_sender.send(cmd).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SafePlayerStateSnapshot {
    pub state: PlayerState,
    pub playlist: Vec<SongInfo>,
    pub current_index: Option<usize>,
    pub play_mode: PlayMode,
    pub volume: f32, // Added volume
}

/// 在独立线程中运行播放器
/// 此函数处理所有与rodio相关的操作，确保线程安全
fn run_player_thread(
    mut cmd_rx: mpsc::Receiver<PlayerCommand>,
    event_tx: mpsc::Sender<PlayerEvent>,
    state: Arc<Mutex<SafePlayerState>>,
    command_sender_for_internal_use: mpsc::Sender<PlayerCommand>, // For sending commands like auto-next
) -> anyhow::Result<()> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let mut current_sink: Option<rodio::Sink> = None;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let player_thread_event_tx = event_tx.clone();

    runtime.block_on(async move {
        let mut progress_interval = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(cmd) = cmd_rx.recv() => {
                    let mut player_state_guard = state.lock().unwrap();

                    match cmd {
                        PlayerCommand::Play => {
                            match player_state_guard.state {
                                PlayerState::Paused => {
                                    if let Some(sink) = &current_sink {
                                        sink.play();
                                        player_state_guard.state = PlayerState::Playing;
                                        let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                                    }
                                }
                                _ => { // Stopped or new play
                                    if player_state_guard.playlist.is_empty() {
                                        let _ = player_thread_event_tx.try_send(PlayerEvent::Error("播放列表为空".to_string()));
                                        continue;
                                    }

                                    let index = match player_state_guard.current_index {
                                        Some(idx) if idx < player_state_guard.playlist.len() => idx,
                                        _ => 0,
                                    };
                                    player_state_guard.current_index = Some(index);

                                    let song = player_state_guard.playlist[index].clone();
                                    drop(player_state_guard); // Release lock before IO

                                    match std::fs::File::open(&song.path) {
                                        Ok(file) => {
                                            match rodio::Decoder::new(std::io::BufReader::new(file)) {
                                                Ok(source) => {
                                                    if let Some(sink) = current_sink.take() { 
                                                        sink.stop();
                                                    }
                                                    match rodio::Sink::try_new(&stream_handle) {
                                                        Ok(sink) => {
                                                            sink.append(source);
                                                            sink.play();
                                                            current_sink = Some(sink);

                                                            let mut player_state_guard = state.lock().unwrap(); 
                                                            player_state_guard.state = PlayerState::Playing;
                                                            let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                                                            let _ = player_thread_event_tx.try_send(PlayerEvent::SongChanged(index, song.clone()));
                                                        }
                                                        Err(e) => {
                                                            let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法创建音频sink: {}", e)));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("解码音频文件失败: {}", e)));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法打开音频文件: {}", e)));
                                        }
                                    }
                                }
                            }
                        }
                        PlayerCommand::Pause => {
                            if let Some(sink) = &current_sink {
                                sink.pause();
                                player_state_guard.state = PlayerState::Paused;
                                let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                            }
                        }
                        PlayerCommand::Stop => {
                            if let Some(sink) = current_sink.take() { 
                                sink.stop();
                            }
                            player_state_guard.state = PlayerState::Stopped;
                            // player_state_guard.current_index = None; // Optionally reset index on stop
                            let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                        }
                        PlayerCommand::Next | PlayerCommand::Previous => {
                            if player_state_guard.playlist.is_empty() {
                                let _ = player_thread_event_tx.try_send(PlayerEvent::Error("播放列表为空".to_string()));
                                continue;
                            }

                            let current_idx_opt = player_state_guard.current_index;
                            let playlist_len = player_state_guard.playlist.len();
                            let play_mode = player_state_guard.play_mode;

                            let new_index = match cmd {
                                PlayerCommand::Next => match (current_idx_opt, play_mode) {
                                    (Some(idx), PlayMode::Sequential) => if idx + 1 >= playlist_len { 0 } else { idx + 1 },
                                    (Some(idx), PlayMode::Repeat) => idx,
                                    (Some(_), PlayMode::Shuffle) => rand::thread_rng().gen_range(0..playlist_len),
                                    (None, _) => 0,
                                },
                                PlayerCommand::Previous => match (current_idx_opt, play_mode) {
                                    (Some(idx), PlayMode::Sequential) => if idx == 0 { playlist_len.saturating_sub(1) } else { idx - 1 },
                                    (Some(idx), PlayMode::Repeat) => idx,
                                    (Some(_), PlayMode::Shuffle) => rand::thread_rng().gen_range(0..playlist_len),
                                    (None, _) => playlist_len.saturating_sub(1),
                                },
                                _ => unreachable!(),
                            };

                            if playlist_len == 0 { // Should be caught by is_empty earlier, but as a safeguard
                                player_state_guard.current_index = None;
                                if let Some(sink) = current_sink.take() {
                                    sink.stop();
                                }
                                player_state_guard.state = PlayerState::Stopped;
                                let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                                continue;
                            }

                            player_state_guard.current_index = Some(new_index);
                            let song = player_state_guard.playlist[new_index].clone();
                            drop(player_state_guard); 

                            if let Some(sink) = current_sink.take() {
                                sink.stop();
                            }

                            match std::fs::File::open(&song.path) {
                                Ok(file) => match rodio::Decoder::new(std::io::BufReader::new(file)) {
                                    Ok(source) => match rodio::Sink::try_new(&stream_handle) {
                                        Ok(sink) => {
                                            sink.append(source);
                                            sink.play();
                                            current_sink = Some(sink);

                                            let mut player_state_guard = state.lock().unwrap(); 
                                            player_state_guard.state = PlayerState::Playing;
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::SongChanged(new_index, song.clone()));
                                        }
                                        Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法创建音频sink: {}", e))); }
                                    },
                                    Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("解码音频文件失败: {}", e))); }
                                },
                                Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法打开音频文件: {}", e))); }
                            }
                        }
                        PlayerCommand::SetSong(index) => {
                            if index >= player_state_guard.playlist.len() {
                                let _ = player_thread_event_tx.try_send(PlayerEvent::Error("无效的歌曲索引".to_string()));
                                continue;
                            }
                            player_state_guard.current_index = Some(index);
                            let song = player_state_guard.playlist[index].clone();
                            drop(player_state_guard); 

                            if let Some(sink) = current_sink.take() {
                                sink.stop();
                            }
                             match std::fs::File::open(&song.path) {
                                Ok(file) => match rodio::Decoder::new(std::io::BufReader::new(file)) {
                                    Ok(source) => match rodio::Sink::try_new(&stream_handle) {
                                        Ok(sink) => {
                                            sink.append(source);
                                            sink.play();
                                            current_sink = Some(sink);

                                            let mut player_state_guard = state.lock().unwrap(); 
                                            player_state_guard.state = PlayerState::Playing;
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::SongChanged(index, song.clone()));
                                        }
                                        Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法创建音频sink: {}", e))); }
                                    },
                                    Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("解码音频文件失败: {}", e))); }
                                },
                                Err(e) => { let _ = player_thread_event_tx.try_send(PlayerEvent::Error(format!("无法打开音频文件: {}", e))); }
                            }
                        }
                        PlayerCommand::AddSongs(songs) => {
                            for song in songs {
                                player_state_guard.playlist.push(song);
                            }
                            if player_state_guard.current_index.is_none() && !player_state_guard.playlist.is_empty() {
                                player_state_guard.current_index = Some(0);
                            }
                            let _ = player_thread_event_tx.try_send(PlayerEvent::PlaylistUpdated(player_state_guard.playlist.clone()));
                        }
                        PlayerCommand::AddSong(song_info) => {
                            player_state_guard.playlist.push(song_info.clone());
                            if player_state_guard.playlist.len() == 1 {
                                player_state_guard.current_index = Some(0);
                            }
                            let _ = player_thread_event_tx.try_send(PlayerEvent::PlaylistUpdated(player_state_guard.playlist.clone()));
                        }
                        PlayerCommand::RemoveSong(index) => {
                            if index >= player_state_guard.playlist.len() {
                                let _ = player_thread_event_tx.try_send(PlayerEvent::Error("无效的歌曲索引".to_string()));
                                continue;
                            }
                            player_state_guard.playlist.remove(index);

                            let mut stopped_playing = false;
                            if let Some(current_idx) = player_state_guard.current_index {
                                if index == current_idx {
                                    if let Some(sink) = current_sink.take() {
                                        sink.stop();
                                    }
                                    // Simplified logic for updating current_index
                                    if !player_state_guard.playlist.is_empty() {
                                        let new_playlist_len = player_state_guard.playlist.len();
                                        // If current_idx was valid for the old list,
                                        // it's either still valid for the new list (items shifted),
                                        // or it was the last item and now needs to point to the new last item.
                                        let new_idx = if current_idx >= new_playlist_len {
                                            new_playlist_len.saturating_sub(1)
                                        } else {
                                            current_idx
                                        };
                                        player_state_guard.current_index = Some(new_idx);
                                    } else {
                                        player_state_guard.current_index = None;
                                    }
                                    player_state_guard.state = PlayerState::Stopped;
                                    stopped_playing = true;
                                } else if index < current_idx {
                                    player_state_guard.current_index = Some(current_idx - 1);
                                }
                            }
                            let playlist_clone = player_state_guard.playlist.clone();
                            let current_state = player_state_guard.state;
                            drop(player_state_guard);

                            if stopped_playing {
                                let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(current_state));
                            }
                            let _ = player_thread_event_tx.try_send(PlayerEvent::PlaylistUpdated(playlist_clone));
                        }
                        PlayerCommand::ClearPlaylist => {
                            if let Some(sink) = current_sink.take() {
                                sink.stop();
                            }
                            player_state_guard.playlist.clear();
                            player_state_guard.current_index = None;
                            player_state_guard.state = PlayerState::Stopped;
                            let _ = player_thread_event_tx.try_send(PlayerEvent::StateChanged(player_state_guard.state));
                            let _ = player_thread_event_tx.try_send(PlayerEvent::PlaylistUpdated(player_state_guard.playlist.clone()));
                        }
                        PlayerCommand::SetPlayMode(mode) => {
                            player_state_guard.play_mode = mode;
                        }
+                       PlayerCommand::SetVolume(vol) => {
+                            player_state_guard.volume = vol;
+                            if let Some(sink) = &current_sink {
+                                sink.set_volume(vol);
+                            }
+                        }
+                       PlayerCommand::SeekTo(position_secs) => {
+                            // Seek functionality not supported with current rodio Sink
+                            let _ = player_thread_event_tx.try_send(PlayerEvent::Error("Seek not implemented".to_string()));
+                        }
                    }
                }
                _ = progress_interval.tick() => {
                    let player_state_guard = state.lock().unwrap(); 
                    if player_state_guard.state == PlayerState::Playing {
                        if let Some(sink) = &current_sink {
                            if sink.empty() { // Song finished
                                if player_state_guard.current_index.is_some() && !player_state_guard.playlist.is_empty() {
                                    drop(player_state_guard); // Release lock before sending command
                                    if command_sender_for_internal_use.try_send(PlayerCommand::Next).is_err() {
                                        eprintln!("播放器线程: 无法发送内部 Next 命令 (通道已满或已关闭)");
                                    }
                                }
                            } else {
                                // Progress update
                                if let Some(idx) = player_state_guard.current_index {
                                    if let Some(song) = player_state_guard.playlist.get(idx) {
                                        if let Some(duration) = song.duration {
                                            // Position calculation is still a challenge with rodio 0.17 Sink.
                                            // For now, sending a placeholder or a rough estimate.
                                            // rodio::Sink::len() is samples remaining. rodio::Sink::elapsed() is not available.
                                            // A true position requires more complex tracking or a library update.
                                            let position: u64 = 0; // Placeholder for actual position
                                            let _ = player_thread_event_tx.try_send(PlayerEvent::ProgressUpdate { position, duration });
                                        }
                                    }
                                }
                            }
                        }
                    } else if player_state_guard.state == PlayerState::Stopped && current_sink.is_some(){
                        // If state is stopped but sink exists, means it was stopped externally, clear sink
                        drop(player_state_guard);
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                    }
                }
                else => {
                    break; 
                }
            }
        }
    });

    Ok(())
}
