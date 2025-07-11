<script setup lang="ts">
import { computed } from 'vue';
import { usePlayerStore, MediaType } from '../stores/player';

// 使用播放器状态
const playerStore = usePlayerStore();

// 计算当前歌曲是否支持模式切换
const currentSongSupportsModeSwitch = computed(() => {
  const currentSong = playerStore.currentSong;
  if (!currentSong) return false;
  
  // 如果歌曲有MV路径，支持音频/视频模式切换
  return !!currentSong.mvPath;
});

// 切换播放模式
const togglePlaybackMode = async () => {
  if (!currentSongSupportsModeSwitch.value) {
    console.log('当前歌曲不支持模式切换');
    return;
  }
  
  try {
    const newMode = playerStore.currentPlaybackMode === MediaType.Audio 
      ? MediaType.Video 
      : MediaType.Audio;
    
    console.log('切换播放模式:', playerStore.currentPlaybackMode, '->', newMode);
    await playerStore.setPlaybackMode(newMode);
  } catch (error) {
    console.error('切换播放模式失败:', error);
  }
};

// 获取模式信息
const getModeInfo = (mode: MediaType) => {
  switch (mode) {
    case MediaType.Audio:
      return {
        icon: '🎵',
        text: '音频',
        description: '播放音频文件'
      };
    case MediaType.Video:
      return {
        icon: '🎬',
        text: '视频',
        description: '播放MV视频'
      };
    default:
      return {
        icon: '🎵',
        text: '音频',
        description: '播放音频文件'
      };
  }
};

const audioModeInfo = computed(() => getModeInfo(MediaType.Audio));
const videoModeInfo = computed(() => getModeInfo(MediaType.Video));
</script>

<template>
  <div class="playback-mode-control" v-if="currentSongSupportsModeSwitch">
    <div class="mode-buttons">
      <button
        class="mode-btn"
        :class="{ 
          'active': playerStore.currentPlaybackMode === 'Audio',
          'inactive': playerStore.currentPlaybackMode !== 'Audio'
        }"
        @click="playerStore.setPlaybackMode(MediaType.Audio)"
        :title="audioModeInfo.description"
      >
        <span class="mode-icon">{{ audioModeInfo.icon }}</span>
        <span class="mode-text">{{ audioModeInfo.text }}</span>
      </button>
      
      <button
        class="mode-btn"
        :class="{ 
          'active': playerStore.currentPlaybackMode === 'Video',
          'inactive': playerStore.currentPlaybackMode !== 'Video'
        }"
        @click="playerStore.setPlaybackMode(MediaType.Video)"
        :title="videoModeInfo.description"
      >
        <span class="mode-icon">{{ videoModeInfo.icon }}</span>
        <span class="mode-text">{{ videoModeInfo.text }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.playback-mode-control {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 0.75rem;
}

.mode-buttons {
  display: flex;
  gap: 0.5rem;
  background: var(--background-glass);
  padding: 0.25rem;
  border-radius: var(--radius-md);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border-light);
}

.mode-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  font-size: 0.8rem;
  font-weight: 600;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
  min-width: 70px;
}

.mode-btn.active {
  background: var(--primary-gradient);
  color: white;
  box-shadow: var(--shadow-primary);
  transform: translateY(-1px);
}

.mode-btn.inactive {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid transparent;
}

.mode-btn.inactive:hover {
  background: var(--background-secondary);
  color: var(--text-primary);
  border-color: var(--border-primary);
  transform: translateY(-1px);
}

.mode-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  transition: left var(--transition-slow);
}

.mode-btn:hover::before {
  left: 100%;
}

.mode-icon {
  font-size: 1rem;
  transition: transform var(--transition-normal);
}

.mode-btn.active .mode-icon {
  transform: scale(1.1);
}

.mode-text {
  white-space: nowrap;
  font-size: 0.75rem;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .mode-btn {
    padding: 0.4rem 0.7rem;
    font-size: 0.75rem;
    min-width: 60px;
  }
  
  .mode-icon {
    font-size: 0.9rem;
  }
  
  .mode-text {
    font-size: 0.7rem;
  }
}
</style>