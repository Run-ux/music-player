<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 播放模式枚举
enum PlayMode {
  Sequential = 'Sequential',
  Repeat = 'Repeat', 
  Shuffle = 'Shuffle'
}

const currentMode = ref<PlayMode>(PlayMode.Sequential);

// 计算播放模式显示信息
const modeInfo = computed(() => {
  switch (currentMode.value) {
    case PlayMode.Sequential:
      return {
        icon: '🔁',
        text: '顺序播放',
        description: '按顺序播放所有歌曲'
      };
    case PlayMode.Repeat:
      return {
        icon: '🔂',
        text: '单曲循环',
        description: '重复播放当前歌曲'
      };
    case PlayMode.Shuffle:
      return {
        icon: '🔀',
        text: '随机播放',
        description: '随机播放播放列表中的歌曲'
      };
    default:
      return {
        icon: '🔁',
        text: '顺序播放',
        description: '按顺序播放所有歌曲'
      };
  }
});

// 切换播放模式
const togglePlayMode = async () => {
  const modes = [PlayMode.Sequential, PlayMode.Repeat, PlayMode.Shuffle];
  const currentIndex = modes.indexOf(currentMode.value);
  const nextIndex = (currentIndex + 1) % modes.length;
  const newMode = modes[nextIndex];
  
  try {
    await invoke('set_play_mode', { mode: newMode });
    currentMode.value = newMode;
    console.log('播放模式已切换到:', newMode);
  } catch (error) {
    console.error('切换播放模式失败:', error);
  }
};

// 初始化时获取当前播放模式
onMounted(async () => {
  try {
    const mode = await invoke('get_play_mode') as PlayMode;
    currentMode.value = mode;
  } catch (error) {
    console.error('获取播放模式失败:', error);
  }
});
</script>

<template>
  <div class="play-mode-control">
    <button 
      @click="togglePlayMode" 
      class="mode-button btn btn-secondary"
      :class="{ 
        'mode-sequential': currentMode === 'Sequential',
        'mode-repeat': currentMode === 'Repeat',
        'mode-shuffle': currentMode === 'Shuffle'
      }"
      :title="modeInfo.description"
    >
      <span class="mode-icon">{{ modeInfo.icon }}</span>
      <span class="mode-text">{{ modeInfo.text }}</span>
    </button>
  </div>
</template>

<style scoped>
.play-mode-control {
  display: flex;
  align-items: center;
  justify-content: center;
}

.mode-button {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  min-width: 140px;
  padding: 0.75rem 1rem;
  font-size: 0.85rem;
  position: relative;
  overflow: hidden;
}

.mode-button::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  opacity: 0;
  transition: opacity var(--transition-normal);
  border-radius: var(--radius-md);
}

.mode-button.mode-sequential::after {
  background: linear-gradient(135deg, rgba(33, 150, 243, 0.15), rgba(33, 150, 243, 0.1));
}

.mode-button.mode-repeat::after {
  background: linear-gradient(135deg, rgba(255, 152, 0, 0.15), rgba(255, 152, 0, 0.1));
}

.mode-button.mode-shuffle::after {
  background: linear-gradient(135deg, rgba(156, 39, 176, 0.15), rgba(156, 39, 176, 0.1));
}

.mode-button:hover::after {
  opacity: 1;
}

.mode-button.mode-sequential {
  border-color: rgba(33, 150, 243, 0.3);
  color: #2196F3;
}

.mode-button.mode-repeat {
  border-color: rgba(255, 152, 0, 0.3);
  color: #FF9800;
}

.mode-button.mode-shuffle {
  border-color: rgba(156, 39, 176, 0.3);
  color: #9C27B0;
}

.mode-button:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.mode-button.mode-sequential:hover {
  border-color: #2196F3;
  box-shadow: 0 4px 16px rgba(33, 150, 243, 0.2);
}

.mode-button.mode-repeat:hover {
  border-color: #FF9800;
  box-shadow: 0 4px 16px rgba(255, 152, 0, 0.2);
}

.mode-button.mode-shuffle:hover {
  border-color: #9C27B0;
  box-shadow: 0 4px 16px rgba(156, 39, 176, 0.2);
}

.mode-icon {
  font-size: 1.1rem;
  position: relative;
  z-index: 2;
  transition: transform var(--transition-normal);
}

.mode-button:hover .mode-icon {
  transform: scale(1.1);
}

.mode-text {
  font-weight: 600;
  white-space: nowrap;
  position: relative;
  z-index: 2;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .mode-button {
    min-width: 120px;
    padding: 0.625rem 0.875rem;
    font-size: 0.8rem;
  }
  
  .mode-icon {
    font-size: 1rem;
  }
}
</style>