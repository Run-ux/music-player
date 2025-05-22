<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';

import { SongInfo } from '../stores/player';

const props = defineProps<{
  currentSong: SongInfo | null;
  isPlaying: boolean;
}>();

const emit = defineEmits<{
  play: [];
  pause: [];
  stop: [];
  next: [];
  previous: [];
}>();

const position = ref(0);
const duration = ref(0);
const progress = ref(0);

// 播放控制
const handlePlay = () => {
  emit('play');
};

const handlePause = () => {
  emit('pause');
};

const handleStop = () => {
  emit('stop');
};

const handleNext = () => {
  emit('next');
};

const handlePrevious = () => {
  emit('previous');
};

// 格式化时间
const formatTime = (seconds: number) => {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
};

// 监听进度更新事件
onMounted(async () => {
  await listen('player-event', (event: any) => {
    const payload = event.payload;
    if (payload.ProgressUpdate) {
      position.value = payload.ProgressUpdate.position;
      duration.value = payload.ProgressUpdate.duration;
      progress.value = (position.value / duration.value) * 100;
    }
  });
});

// 监听当前歌曲变化
watch(() => props.currentSong, (newSong) => {
  if (newSong) {
    position.value = 0;
    duration.value = newSong.duration || 0;
    progress.value = 0;
  }
}, { deep: true });
</script>

<template>
  <div class="player-controls">
    <div class="progress-bar">
      <div class="progress-info">
        <span>{{ formatTime(position) }}</span>
        <span>{{ formatTime(duration) }}</span>
      </div>
      <div class="progress-container">
        <div class="progress" :style="{ width: `${progress}%` }"></div>
      </div>
    </div>
    
    <div class="control-buttons">
      <button @click="handlePrevious" class="control-btn">
        <i class="icon-previous">⏮</i>
      </button>
      <button v-if="!isPlaying" @click="handlePlay" class="control-btn play">
        <i class="icon-play">▶</i>
      </button>
      <button v-else @click="handlePause" class="control-btn pause">
        <i class="icon-pause">⏸</i>
      </button>
      <button @click="handleStop" class="control-btn">
        <i class="icon-stop">⏹</i>
      </button>
      <button @click="handleNext" class="control-btn">
        <i class="icon-next">⏭</i>
      </button>
    </div>
  </div>
</template>

<style scoped>
.player-controls {
  padding: 1rem;
  border-radius: 8px;
  background: #f5f5f5;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.progress-bar {
  margin-bottom: 1rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
  font-size: 0.85rem;
  color: #555;
}

.progress-container {
  height: 6px;
  background-color: #ddd;
  border-radius: 3px;
  overflow: hidden;
}

.progress {
  height: 100%;
  background-color: #4caf50;
  transition: width 0.3s;
}

.control-buttons {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}

.control-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.5rem;
  color: #333;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.control-btn:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.play, .pause {
  font-size: 1.8rem;
  background-color: #e0f7e0;
  border: 1px solid #4caf50;
  color: #4caf50;
}

.play:hover, .pause:hover {
  background-color: #d0f0d0;
}
</style>
