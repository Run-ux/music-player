use crate::player_fixed::PlayerEvent;
use crate::player_safe::SafePlayerManager as AudioPlayer;
use std::sync::Mutex as StdMutex;
use std::sync::{Arc, Once};
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;

// 播放器包装器
pub struct PlayerWrapper {
    pub player: Arc<AudioPlayer>,
}

// 全局单例模式
pub struct GlobalPlayer {
    player: Option<Arc<AsyncMutex<PlayerWrapper>>>,
    event_rx: StdMutex<Option<mpsc::Receiver<PlayerEvent>>>,
    initialized: bool,
}

// 安全的单例访问
impl GlobalPlayer {
    // 获取单例实例
    pub fn instance() -> &'static StdMutex<GlobalPlayer> {
        static INIT: Once = Once::new();
        static mut INSTANCE: Option<StdMutex<GlobalPlayer>> = None;

        INIT.call_once(|| unsafe {
            INSTANCE = Some(StdMutex::new(GlobalPlayer {
                player: None,
                event_rx: StdMutex::new(None),
                initialized: false,
            }));
        });

        unsafe { INSTANCE.as_ref().unwrap() }
    } // 初始化播放器
    pub fn initialize(&mut self) -> (Arc<AsyncMutex<PlayerWrapper>>, mpsc::Receiver<PlayerEvent>) {
        if !self.initialized {
            // 创建新的播放器实例
            let (audio_player, event_rx) = AudioPlayer::new();

            // 创建包装器
            let player_wrapper = PlayerWrapper {
                player: Arc::new(audio_player),
            };

            self.player = Some(Arc::new(AsyncMutex::new(player_wrapper)));
            *self.event_rx.lock().unwrap() = Some(event_rx);
            self.initialized = true;
        }

        // 返回播放器引用和事件接收器
        let player = self.player.as_ref().unwrap().clone();
        let mut event_rx_guard = self.event_rx.lock().unwrap();
        let event_rx = event_rx_guard
            .take()
            .expect("Event receiver was already taken");

        (player, event_rx)
    }

    // 获取播放器引用
    pub fn get_player(&self) -> Option<Arc<AsyncMutex<PlayerWrapper>>> {
        self.player.clone()
    }

    // 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
