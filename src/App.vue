<script setup lang="ts">
import { onMounted, computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { usePlayerStore, PlayerState, SongInfo, MediaType } from "./stores/player";
import PlayerControls from "./components/PlayerControls.vue";
import Playlist from "./components/Playlist.vue";
import NowPlaying from "./components/NowPlaying.vue";
import LyricsDisplay from "./components/LyricsDisplay.vue";
import VideoPlayer from "./components/VideoPlayer.vue";

// 使用播放器状态
const playerStore = usePlayerStore();

// 决定是否显示视频播放器
const shouldShowVideo = computed(() => {
  const currentSong = playerStore.currentSong;
  if (!currentSong) return false;
  
  // 如果当前播放模式是Video，且当前歌曲有MV，则显示视频
  if (playerStore.currentPlaybackMode === MediaType.Video && currentSong.mvPath) {
    return true;
  }
  
  // 如果当前歌曲本身就是视频文件，则显示视频
  if (currentSong.mediaType === MediaType.Video) {
    return true;
  }
  
  return false;
});

// 决定是否显示歌词
const shouldShowLyrics = computed(() => {
  const currentSong = playerStore.currentSong;
  if (!currentSong) return false;
  
  // 如果正在显示视频，不显示歌词
  if (shouldShowVideo.value) return false;
  
  // 如果是音频模式或没有MV，显示歌词
  return playerStore.currentPlaybackMode === MediaType.Audio || !currentSong.mvPath;
});

// 初始化播放器
const initPlayer = async () => {  
  try {
    await invoke('init_player');
    
    // 获取播放列表
    const playlist = await invoke('get_playlist') as SongInfo[];
    playerStore.updatePlaylist(playlist);
    
    // 获取当前索引
    const currentIndex = await invoke('get_current_index') as number | null;
    if (currentIndex !== null) {
      playerStore.updateCurrentSong(currentIndex);
    }
    
    // 获取播放器状态
    const state = await invoke('get_player_state');
    playerStore.updateState(state as PlayerState);
    
    // 监听歌曲添加事件
    await listen('songs_added', async () => {
      try {
        const updatedPlaylist = await invoke('get_playlist') as SongInfo[];
        playerStore.updatePlaylist(updatedPlaylist);
      } catch (error) {
        console.error('Error refreshing playlist after songs_added:', error);
      }
    });
      // 监听播放器事件
    await listen('player-event', (event: any) => {
      const payload = event.payload;
      
      // 新的事件格式：使用 tag 和 content 
      if (payload.type && payload.data !== undefined) {
          switch (payload.type) {
          case 'StateChanged':
            playerStore.updateState(payload.data);
            break;
            
          case 'SongChanged':
            playerStore.updateCurrentSong(payload.data[0]);
            // 切换歌曲时重置进度条
            playerStore.updateProgress(0, payload.data[1]?.duration || 0);
            break;
              case 'PlaylistUpdated':
            if (Array.isArray(payload.data)) {
              playerStore.updatePlaylist(payload.data);
            } else {
              console.error('Playlist data is not an array:', payload.data);
            }
            break;
              case 'ProgressUpdate':
            if (payload.data && typeof payload.data === 'object') {
              playerStore.updateProgress(
                payload.data.position,
                payload.data.duration
              );
            }
            break;
            
          case 'Error':
            console.error('播放器错误:', payload.data);
            break;
            
          default:
            console.warn('Unknown event type:', payload.type);        }
      } else {
        // 兼容旧格式
        if (payload.StateChanged) {
          playerStore.updateState(payload.StateChanged);
        }

        if (payload.SongChanged) {
          playerStore.updateCurrentSong(payload.SongChanged[0]);
          // 切换歌曲时重置进度条
          playerStore.updateProgress(0, payload.SongChanged[1]?.duration || 0);
        }
          if (payload.PlaylistUpdated) {
          playerStore.updatePlaylist(payload.PlaylistUpdated);
        }
        
        if (payload.ProgressUpdate) {
          playerStore.updateProgress(
            payload.ProgressUpdate.position,
            payload.ProgressUpdate.duration
          );
        }
        
        if (payload.Error) {
          console.error('播放器错误 (legacy):', payload.Error);        }      }
    });
    
  } catch (error) {
    console.error('初始化播放器失败:', error);
  }
};

// 播放控制
const handlePlay = () => {
  playerStore.play();
};

const handlePause = () => {
  playerStore.pause();
};

const handleNext = () => {
  playerStore.next();
};

const handlePrevious = () => {
  playerStore.previous();
};

const handleSelectSong = async (index: number) => {
  // 如果点击的是当前正在播放的歌曲，切换播放/暂停状态
  if (playerStore.currentIndex === index && playerStore.isPlaying) {
    await playerStore.pause();
  } else {
    // 设置当前歌曲（这已经会自动开始播放）
    await playerStore.setCurrentSong(index);
    // 注意：不需要再次调用 play()，因为 setCurrentSong 已经设置为播放状态
  }
};

const handleRemoveSong = (index: number) => {
  playerStore.removeSong(index);
};

// 歌词跳转处理
const handleLyricsSeek = (time: number) => {
  playerStore.seekTo(time);
};

// 组件挂载时初始化
onMounted(() => {
  initPlayer();
  
  // 初始化播放模式
  playerStore.initializePlaybackMode();
  
  // 添加默认专辑图片
  const link = document.createElement('link');
  link.rel = 'preload';
  link.href = '/default-album.png';
  link.as = 'image';
  document.head.appendChild(link);
});


const showSplash = ref(true);

// 模拟加载过程
setTimeout(() => {
  showSplash.value = false;
}, 5000);
</script>

<template>
  <div class="app">
    <!-- 启动动画屏幕 -->
    <div v-if="showSplash" class="splash-screen">
      <div class="splash-content">
        <!-- Logo区域 -->
        <div class="logo-container">
          <div class="logo-icon">🎵</div>
          <div class="logo-rings">
            <div class="ring ring-1"></div>
            <div class="ring ring-2"></div>
            <div class="ring ring-3"></div>
          </div>
        </div>
        
        <!-- 项目标题 -->
        <div class="project-title">
          <div class="title-main">Rust大作业</div>
          <div class="title-divider"></div>
          <div class="title-sub">音乐播放器</div>
        </div>
        
        <!-- 加载指示器 -->
        <div class="loading-indicator">
          <div class="loading-dots">
            <div class="dot"></div>
            <div class="dot"></div>
            <div class="dot"></div>
          </div>
          <div class="loading-text">正在启动...</div>
        </div>
      </div>
      
      <!-- 背景装饰 -->
      <div class="splash-bg-decoration">
        <div class="floating-note note-1">♪</div>
        <div class="floating-note note-2">♫</div>
        <div class="floating-note note-3">♩</div>
        <div class="floating-note note-4">♬</div>
        <div class="floating-note note-5">♭</div>
        <div class="floating-note note-6">♯</div>
      </div>
    </div>

    <!-- 主应用内容 -->
    <div v-else class="main-app">
      <header class="app-header animate-slide-down">
        <h1>音乐播放器</h1>
      </header>
      
      <main class="app-content">
        <div class="left-panel animate-slide-right">
          <NowPlaying 
            :song="playerStore.currentSong" 
            :is-playing="playerStore.isPlaying" 
          />
          <PlayerControls 
            :current-song="playerStore.currentSong"
            :is-playing="playerStore.isPlaying"
            @play="handlePlay"
            @pause="handlePause"
            @next="handleNext"
            @previous="handlePrevious"
          />
        </div>
        
        <div class="center-panel animate-fade-in">
          <!-- 根据播放模式和媒体类型显示不同组件 -->
          <VideoPlayer
            v-if="shouldShowVideo"
            :song="playerStore.currentSong"
            :is-playing="playerStore.isPlaying"
          />
          <LyricsDisplay
            v-else-if="shouldShowLyrics"
            :lyrics="playerStore.currentSong?.lyrics"
            :current-time="playerStore.position"
            :is-playing="playerStore.isPlaying"
            @seek="handleLyricsSeek"
          />
          <div v-else class="no-content-placeholder">
            <div class="placeholder-icon">🎵</div>
            <p>选择歌曲开始播放</p>
          </div>
        </div>
        
        <div class="right-panel animate-slide-left">
          <Playlist 
            :playlist="playerStore.playlist" 
            :current-index="playerStore.currentIndex"
            :is-playing="playerStore.isPlaying"
            @select-song="handleSelectSong"
            @remove-song="handleRemoveSong"
          />
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  z-index: 1;
}

.app > * {
  position: relative;
  z-index: 2;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* 背景装饰音符 */
.splash-bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: 1;
}

.floating-note {
  position: absolute;
  font-size: 2rem;
  color: rgba(255, 255, 255, 0.1);
  animation: floatNote 8s ease-in-out infinite;
}

.note-1 {
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.note-2 {
  top: 20%;
  right: 15%;
  animation-delay: 1s;
}

.note-3 {
  top: 60%;
  left: 5%;
  animation-delay: 2s;
}

.note-4 {
  bottom: 20%;
  right: 10%;
  animation-delay: 3s;
}

.note-5 {
  top: 40%;
  left: 80%;
  animation-delay: 4s;
}

.note-6 {
  bottom: 40%;
  left: 20%;
  animation-delay: 5s;
}

@keyframes floatNote {
  0%, 100% {
    transform: translateY(0px) rotate(0deg);
    opacity: 0.1;
  }
  50% {
    transform: translateY(-20px) rotate(180deg);
    opacity: 0.3;
  }
}

.main-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* 背景装饰音符 */
.splash-bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: 1;
}

.floating-note {
  position: absolute;
  font-size: 2rem;
  color: rgba(255, 255, 255, 0.1);
  animation: floatNote 8s ease-in-out infinite;
}

.note-1 {
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.note-2 {
  top: 20%;
  right: 15%;
  animation-delay: 1s;
}

.note-3 {
  top: 60%;
  left: 5%;
  animation-delay: 2s;
}

.note-4 {
  bottom: 20%;
  right: 10%;
  animation-delay: 3s;
}

.note-5 {
  top: 40%;
  left: 80%;
  animation-delay: 4s;
}

.note-6 {
  bottom: 40%;
  left: 20%;
  animation-delay: 5s;
}

@keyframes floatNote {
  0%, 100% {
    transform: translateY(0px) rotate(0deg);
    opacity: 0.1;
  }
  50% {
    transform: translateY(-20px) rotate(180deg);
    opacity: 0.3;
  }
}

.main-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* 背景装饰音符 */
.splash-bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: 1;
}

.floating-note {
  position: absolute;
  font-size: 2rem;
  color: rgba(255, 255, 255, 0.1);
  animation: floatNote 8s ease-in-out infinite;
}

.note-1 {
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.note-2 {
  top: 20%;
  right: 15%;
  animation-delay: 1s;
}

.note-3 {
  top: 60%;
  left: 5%;
  animation-delay: 2s;
}

.note-4 {
  bottom: 20%;
  right: 10%;
  animation-delay: 3s;
}

.note-5 {
  top: 40%;
  left: 80%;
  animation-delay: 4s;
}

.note-6 {
  bottom: 40%;
  left: 20%;
  animation-delay: 5s;
}

@keyframes floatNote {
  0%, 100% {
    transform: translateY(0px) rotate(0deg);
    opacity: 0.1;
  }
  50% {
    transform: translateY(-20px) rotate(180deg);
    opacity: 0.3;
  }
}

.main-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* 背景装饰音符 */
.splash-bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: 1;
}

.floating-note {
  position: absolute;
  font-size: 2rem;
  color: rgba(255, 255, 255, 0.1);
  animation: floatNote 8s ease-in-out infinite;
}

.note-1 {
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.note-2 {
  top: 20%;
  right: 15%;
  animation-delay: 1s;
}

.note-3 {
  top: 60%;
  left: 5%;
  animation-delay: 2s;
}

.note-4 {
  bottom: 20%;
  right: 10%;
  animation-delay: 3s;
}

.note-5 {
  top: 40%;
  left: 80%;
  animation-delay: 4s;
}

.note-6 {
  bottom: 40%;
  left: 20%;
  animation-delay: 5s;
}

@keyframes floatNote {
  0%, 100% {
    transform: translateY(0px) rotate(0deg);
    opacity: 0.1;
  }
  50% {
    transform: translateY(-20px) rotate(180deg);
    opacity: 0.3;
  }
}

.main-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* 背景装饰音符 */
.splash-bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: 1;
}

.floating-note {
  position: absolute;
  font-size: 2rem;
  color: rgba(255, 255, 255, 0.1);
  animation: floatNote 8s ease-in-out infinite;
}

.note-1 {
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.note-2 {
  top: 20%;
  right: 15%;
  animation-delay: 1s;
}

.note-3 {
  top: 60%;
  left: 5%;
  animation-delay: 2s;
}

.note-4 {
  bottom: 20%;
  right: 10%;
  animation-delay: 3s;
}

.note-5 {
  top: 40%;
  left: 80%;
  animation-delay: 4s;
}

.note-6 {
  bottom: 40%;
  left: 20%;
  animation-delay: 5s;
}

@keyframes floatNote {
  0%, 100% {
    transform: translateY(0px) rotate(0deg);
    opacity: 0.1;
  }
  50% {
    transform: translateY(-20px) rotate(180deg);
    opacity: 0.3;
  }
}

.main-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  position: relative;
}

.app-header {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 1rem 1.5rem;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  text-align: center;
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.app-header h1 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
  min-height: 0;
}

.left-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  height: 100%;
}

.center-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  min-width: 350px;
}

.right-panel {
  flex: 3;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 250px;
  color: #6b7280;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 2px dashed rgba(102, 126, 234, 0.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: hidden;
}

.no-content-placeholder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  animation: shimmer 3s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 1;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* 启动动画屏幕样式 */
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #667eea 100%);
  background-size: 200% 200%;
  animation: gradientShift 4s ease-in-out infinite;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

.splash-content {
  text-align: center;
  color: white;
  position: relative;
  z-index: 2;
}

.logo-container {
  position: relative;
  margin-bottom: 3rem;
}

.logo-icon {
  font-size: 5rem;
  margin-bottom: 1rem;
  animation: logoFloat 3s ease-in-out infinite;
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.3));
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.logo-rings {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: ringPulse 2s ease-in-out infinite;
}

.ring-1 {
  width: 120px;
  height: 120px;
  margin: -60px 0 0 -60px;
  animation-delay: 0s;
}

.ring-2 {
  width: 160px;
  height: 160px;
  margin: -80px 0 0 -80px;
  animation-delay: 0.5s;
}

.ring-3 {
  width: 200px;
  height: 200px;
  margin: -100px 0 0 -100px;
  animation-delay: 1s;
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.8);
    opacity: 1;
  }
}

.project-title {
  margin-bottom: 4rem;
}

.title-main {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(45deg, #ffffff, #e0e7ff, #ffffff);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: titleShimmer 3s ease-in-out infinite, titleSlideIn 1s ease-out;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

@keyframes titleShimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

@keyframes titleSlideIn {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-divider {
  width: 100px;
  height: 3px;
  background: linear-gradient(90deg, transparent, white, transparent);
  margin: 1rem auto;
  animation: dividerExpand 1.5s ease-out 0.5s forwards;
  transform: scaleX(0);
}

@keyframes dividerExpand {
  to { transform: scaleX(1); }
}

.title-sub {
  font-size: 1.8rem;
  font-weight: 600;
  opacity: 0.9;
  animation: titleSlideIn 1s ease-out 0.7s forwards;
  opacity: 0;
  letter-spacing: 2px;
  margin-bottom: 2rem;
}

.loading-indicator {
  position: fixed;
  bottom: 3rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}
</style>

<style>
:root {
  /* 统一的设计系统变量 */
  --primary-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  --primary-color: #667eea;
  --primary-dark: #5a67d8;
  --secondary-color: #764ba2;
  --accent-color: #4ade80;
  --danger-color: #ef4444;
  --text-primary: #1f2937;
  --text-secondary: #6b7280;
  --text-muted: #9ca3af;
  --background-primary: rgba(255, 255, 255, 0.95);
  --background-secondary: rgba(255, 255, 255, 0.8);
  --background-glass: rgba(255, 255, 255, 0.1);
  --border-light: rgba(255, 255, 255, 0.2);
  --border-primary: rgba(102, 126, 234, 0.3);
  --shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.08);
  --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.12);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.15);
  --shadow-primary: 0 4px 20px rgba(102, 126, 234, 0.3);
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 20px;
  --transition-fast: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-normal: 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 0.5s cubic-bezier(0.4, 0, 0.2, 1);

  font-family: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  font-weight: 400;
  color: var(--text-primary);
  background: var(--primary-gradient);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

/* 统一的按钮样式 */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  font-size: 0.9rem;
  font-weight: 500;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
  text-decoration: none;
  outline: none;
}

.btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  transition: left var(--transition-slow);
}

.btn:hover::before {
  left: 100%;
}

.btn:hover {
  transform: translateY(-2px);
}

.btn:active {
  transform: translateY(0);
}

.btn-primary {
  background: var(--primary-gradient);
  color: white;
  box-shadow: var(--shadow-primary);
}

.btn-primary:hover {
  box-shadow: 0 6px 25px rgba(102, 126, 234, 0.4);
}

.btn-secondary {
  background: var(--background-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
  box-shadow: var(--shadow-sm);
}

.btn-secondary:hover {
  background: var(--background-primary);
  box-shadow: var(--shadow-md);
}

.btn-success {
  background: linear-gradient(135deg, #4ade80, #22c55e);
  color: white;
  box-shadow: 0 4px 15px rgba(34, 197, 94, 0.3);
}

.btn-danger {
  background: linear-gradient(135deg, #f87171, #ef4444);
  color: white;
  box-shadow: 0 4px 15px rgba(239, 68, 68, 0.3);
}

.btn:disabled {
  background: var(--text-muted);
  color: white;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* 统一的卡片样式 */
.card {
  background: var(--background-secondary);
  backdrop-filter: blur(10px);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md);
  position: relative;
  overflow: hidden;
}

.card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  opacity: 0;
  transition: opacity var(--transition-normal);
}

.card:hover::before {
  opacity: 1;
}

/* 统一的输入框样式 */
input, button {
  border-radius: var(--radius-md);
  border: 1px solid var(--border-light);
  padding: 0.75rem 1rem;
  font-size: 0.9rem;
  font-weight: 500;
  font-family: inherit;
  color: var(--text-primary);
  background: var(--background-secondary);
  transition: all var(--transition-fast);
  box-shadow: var(--shadow-sm);
  outline: none;
}

button {
  cursor: pointer;
}

input:focus, button:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

button:hover {
  border-color: var(--primary-color);
  background: var(--background-primary);
}

/* 滚动条样式统一 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: var(--primary-gradient);
  border-radius: 4px;
  transition: background var(--transition-normal);
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #5a67d8, #6b46c1);
}

/* 深色模式支持 */
@media (prefers-color-scheme: dark) {
  :root {
    --text-primary: #f9fafb;
    --text-secondary: #d1d5db;
    --text-muted: #9ca3af;
    --background-primary: rgba(31, 41, 55, 0.95);
    --background-secondary: rgba(31, 41, 55, 0.8);
    --background-glass: rgba(31, 41, 55, 0.3);
    --border-light: rgba(31, 41, 55, 0.3);
  }
}
</style>