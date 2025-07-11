<script setup lang="ts">
import { onMounted, computed } from "vue";
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
</script>

<template>
  <div class="app">
    <header class="app-header">
      <h1>音乐播放器</h1>
    </header>
    
    <main class="app-content">
      <div class="left-panel">
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
      
      <div class="center-panel">
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
      
      <div class="right-panel">
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
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #f8f8f8;
  color: #333;
}

.app-header {
  background-color: #4caf50;
  color: white;
  padding: 1rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  text-align: center;
}

.app-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 500;
}

.app-content {
  display: flex;
  flex: 1;
  padding: 1rem;
  gap: 1rem;
  overflow: hidden;
}

.left-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
}

.center-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 350px;
  max-width: 500px;
}

.right-panel {
  flex: 1;
  min-width: 300px;
}

.no-content-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 300px;
  color: #999;
  background: #f9f9f9;
  border-radius: 8px;
  border: 2px dashed #ddd;
}

.placeholder-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.no-content-placeholder p {
  font-size: 1.1rem;
  margin: 0;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .app-content {
    flex-direction: column;
    overflow-y: auto;
  }
  
  .left-panel,
  .center-panel,
  .right-panel {
    min-width: unset;
    max-width: unset;
  }
  
  .center-panel {
    min-height: 300px;
  }
}

@media (max-width: 768px) {
  .app-content {
    padding: 0.5rem;
    gap: 0.5rem;
  }
  
  .center-panel {
    min-height: 250px;
  }
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>