<script setup lang="ts">
import { onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { usePlayerStore, PlayerState, SongInfo } from "./stores/player";
import PlayerControls from "./components/PlayerControls.vue";
import Playlist from "./components/Playlist.vue";
import NowPlaying from "./components/NowPlaying.vue";

// 使用播放器状态
const playerStore = usePlayerStore();

// 初始化播放器
const initPlayer = async () => {  try {
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
      console.log('Songs added, refreshing playlist');
      const updatedPlaylist = await invoke('get_playlist') as SongInfo[];
      playerStore.updatePlaylist(updatedPlaylist);
    });
    
    // 监听播放器事件
    await listen('player-event', (event: any) => {
      const payload = event.payload;
      
      // 状态变更
      if (payload.StateChanged) {
        playerStore.updateState(payload.StateChanged);
      }
      
      // 歌曲变更
      if (payload.SongChanged) {
        playerStore.updateCurrentSong(payload.SongChanged[0]);
      }
      
      // 播放列表更新
      if (payload.PlaylistUpdated) {
        playerStore.updatePlaylist(payload.PlaylistUpdated);
      }
      
      // 播放进度更新
      if (payload.ProgressUpdate) {
        playerStore.updateProgress(
          payload.ProgressUpdate.position,
          payload.ProgressUpdate.duration
        );
      }
      
      // 错误处理
      if (payload.Error) {
        console.error('播放器错误:', payload.Error);
      }
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

const handleStop = () => {
  playerStore.stop();
};

const handleNext = () => {
  playerStore.next();
};

const handlePrevious = () => {
  playerStore.previous();
};

const handleSelectSong = (index: number) => {
  playerStore.setCurrentSong(index);
};

const handleRemoveSong = (index: number) => {
  playerStore.removeSong(index);
};

// 组件挂载时初始化
onMounted(() => {
  initPlayer();
  
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
          @stop="handleStop"
          @next="handleNext"
          @previous="handlePrevious"
        />
      </div>
      
      <div class="right-panel">
        <Playlist 
          :playlist="playerStore.playlist" 
          :current-index="playerStore.currentIndex"
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
  overflow-y: auto;
}

.right-panel {
  flex: 1;
  overflow: hidden;
  max-width: 400px;
}

@media (max-width: 768px) {
  .app-content {
    flex-direction: column;
  }
  
  .right-panel {
    max-width: none;
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