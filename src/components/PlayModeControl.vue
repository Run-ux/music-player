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
      class="mode-button"
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
}

.mode-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: #f8f9fa;
  color: #333;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s ease;
  min-width: 120px;
  justify-content: center;
}

.mode-button:hover {
  background: #e9ecef;
  border-color: #4caf50;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.mode-button:active {
  transform: translateY(0);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.mode-icon {
  font-size: 1.1rem;
}

.mode-text {
  font-weight: 500;
  white-space: nowrap;
}

/* 不同模式的特殊样式 */
.mode-button[title*="顺序播放"] {
  border-color: #2196F3;
}

.mode-button[title*="单曲循环"] {
  border-color: #FF9800;
}

.mode-button[title*="随机播放"] {
  border-color: #9C27B0;
}

.mode-button[title*="顺序播放"]:hover {
  background: rgba(33, 150, 243, 0.1);
}

.mode-button[title*="单曲循环"]:hover {
  background: rgba(255, 152, 0, 0.1);
}

.mode-button[title*="随机播放"]:hover {
  background: rgba(156, 39, 176, 0.1);
}
</style>