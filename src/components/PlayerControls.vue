<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import PlayModeControl from './PlayModeControl.vue';
import { usePlayerStore, SongInfo, MediaType, PlayerState } from '../stores/player';

const props = defineProps<{
  currentSong: SongInfo | null;
  isPlaying: boolean;
}>();

const emit = defineEmits<{
  play: [];
  pause: [];
  next: [];
  previous: [];
}>();

const position = ref(0);
const duration = ref(0);
const progress = ref(0);
const isUserJumping = ref(false); // 跳转保护标志

// 获取playerStore
const playerStore = usePlayerStore();

// 播放控制 - 合并播放和暂停为一个切换功能
const handlePlayPause = () => {
  console.log('🎮 播放/暂停按钮点击，当前状态:', props.isPlaying);
  
  // 关键修复：立即更新UI状态，避免延迟感
  const willBePlaying = !props.isPlaying;
  
  if (willBePlaying) {
    console.log('🎬 用户点击播放');
    emit('play');
  } else {
    console.log('⏸️ 用户点击暂停');
    emit('pause');
  }
  
  // 新增：对于视频模式，增加额外的状态确认
  const isVideoMode = props.currentSong?.mediaType === 'Video' || 
                     (playerStore.currentPlaybackMode === 'Video' && props.currentSong?.mvPath);
  
  if (isVideoMode) {
    console.log('🎬 视频模式播放控制，确保状态同步');
    // 延迟检查状态一致性
    setTimeout(() => {
      // 检查是否需要状态修正
      if (playerStore.isPlaying !== willBePlaying) {
        console.log('🔧 播放键：检测到状态不一致，可能需要重试');
      }
    }, 200);
  }
};

const handleNext = () => {
  emit('next');
};

const handlePrevious = () => {
  emit('previous');
};

// 进度条点击跳转功能 - 完全重写
const handleProgressClick = (event: MouseEvent) => {
  if (!duration.value || !props.currentSong) return;
  
  const progressContainer = event.currentTarget as HTMLElement;
  const rect = progressContainer.getBoundingClientRect();
  const clickX = event.clientX - rect.left;
  const progressWidth = rect.width;
  
  // 计算点击位置对应的时间
  const clickPercent = Math.max(0, Math.min(1, clickX / progressWidth));
  const targetPosition = Math.floor(clickPercent * duration.value);
  
  console.log(`主进度条点击跳转: ${targetPosition}秒`);
  
  // 关键修复：立即设置保护标志，防止任何干扰
  isUserJumping.value = true;
  
  // 立即更新显示，确保用户看到即时反馈
  position.value = targetPosition;
  progress.value = clickPercent * 100;
  
  // 调用智能跳转 - 这会根据播放模式选择正确的处理方式
  playerStore.seekTo(targetPosition);
  
  // 延长保护时间，确保完全避免干扰
  setTimeout(() => {
    isUserJumping.value = false;
    console.log('跳转保护解除');
  }, 1500);
};

// 进度条拖拽功能
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartProgress = ref(0);

const handleMouseDown = (event: MouseEvent) => {
  if (!duration.value || !props.currentSong) return;
  
  isDragging.value = true;
  dragStartX.value = event.clientX;
  dragStartProgress.value = progress.value;
  
  // 阻止默认行为
  event.preventDefault();
  
  // 添加全局鼠标事件监听器
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const handleMouseMove = (event: MouseEvent) => {
  if (!isDragging.value || !duration.value) return;
  
  const progressContainer = document.querySelector('.progress-container') as HTMLElement;
  if (!progressContainer) return;
  
  const rect = progressContainer.getBoundingClientRect();
  const currentX = event.clientX - rect.left;
  const progressWidth = rect.width;
  
  // 计算新的进度百分比
  const newPercent = Math.max(0, Math.min(100, (currentX / progressWidth) * 100));
  
  // 实时更新进度条显示
  progress.value = newPercent;
  position.value = Math.floor((newPercent / 100) * duration.value);
};

const handleMouseUp = (_event: MouseEvent) => {
  if (!isDragging.value) return;
  
  isDragging.value = false;
  
  // 移除全局事件监听器
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  
  // 执行跳转时也设置保护
  isUserJumping.value = true;
  const targetPosition = Math.floor((progress.value / 100) * duration.value);
  
  console.log(`主进度条拖拽跳转: ${targetPosition}秒`);
  playerStore.seekTo(targetPosition);
  
  // 拖拽后的保护时间
  setTimeout(() => {
    isUserJumping.value = false;
    console.log('拖拽跳转保护解除');
  }, 1200);
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
    // 如果用户正在跳转，忽略后端事件，防止干扰
    if (isUserJumping.value) {
      console.log('用户正在跳转，忽略后端进度事件');
      return;
    }
    
    const payload = event.payload;
    
    // 处理新的事件格式
    if (payload.type === 'ProgressUpdate' && payload.data) {
      position.value = payload.data.position;
      duration.value = payload.data.duration;
      progress.value = duration.value > 0 ? (position.value / duration.value) * 100 : 0;
    }
    // 处理歌曲切换事件，立即重置进度条
    else if (payload.type === 'SongChanged' && payload.data) {
      const [, songInfo] = payload.data;
      position.value = 0;
      duration.value = songInfo?.duration || 0;
      progress.value = 0;
      console.log('歌曲切换，进度条重置:', songInfo?.title);
    }
    // 兼容旧格式
    else if (payload.ProgressUpdate) {
      position.value = payload.ProgressUpdate.position;
      duration.value = payload.ProgressUpdate.duration;
      progress.value = duration.value > 0 ? (position.value / duration.value) * 100 : 0;
    }
    // 兼容旧格式的歌曲切换
    else if (payload.SongChanged) {
      const [, songInfo] = payload.SongChanged;
      position.value = 0;
      duration.value = songInfo?.duration || 0;
      progress.value = 0;
      console.log('歌曲切换(旧格式)，进度条重置:', songInfo?.title);
    }
  });
});

// 新增：直接监听playerStore的进度变化，确保视频模式下进度条也能自动前进
watch(() => [playerStore.position, playerStore.duration], ([newPosition, newDuration]) => {
  // 关键修复：增加更智能的保护逻辑和类型安全
  const shouldUpdateProgress = !isDragging.value && !isUserJumping.value;
  
  if (shouldUpdateProgress && typeof newPosition === 'number' && typeof newDuration === 'number') {
    // 检查是否是视频模式下的自然进度更新
    const isVideoMode = props.currentSong?.mediaType === 'Video' || 
                       (playerStore.currentPlaybackMode === 'Video' && props.currentSong?.mvPath);
    
    if (isVideoMode) {
      // 视频模式：更温和地更新进度条，避免突兀变化
      const oldPosition = position.value;
      const positionDiff = Math.abs(newPosition - oldPosition);
      
      // 如果位置变化很大（可能是跳转），立即更新
      // 如果是小幅变化（正常播放），平滑更新
      if (positionDiff > 2) {
        console.log('PlayerControls: 检测到视频跳转，立即更新进度条:', newPosition);
        position.value = newPosition;
        duration.value = newDuration;
        progress.value = newDuration > 0 ? (newPosition / newDuration) * 100 : 0;
      } else {
        // 平滑更新，避免进度条抖动
        position.value = newPosition;
        duration.value = newDuration;
        progress.value = newDuration > 0 ? (newPosition / newDuration) * 100 : 0;
      }
    } else {
      // 音频模式：正常更新
      position.value = newPosition;
      duration.value = newDuration;
      progress.value = newDuration > 0 ? (newPosition / newDuration) * 100 : 0;
    }
  }
}, { immediate: true });

// 关键修复：单独监听播放状态变化
watch(() => playerStore.state, (newState) => {
  // 确保播放按钮状态与实际播放状态同步
  if (newState === PlayerState.Playing && !props.isPlaying) {
    console.log('🔧 检测到状态不同步：后端播放但前端暂停，可能需要状态修正');
  } else if (newState === PlayerState.Paused && props.isPlaying) {
    console.log('🔧 检测到状态不同步：后端暂停但前端播放，可能需要状态修正');
  }
});

// 监听当前歌曲变化，确保props变化时也重置进度条
watch(() => props.currentSong, (newSong, oldSong) => {
  if (newSong && (!oldSong || newSong.path !== oldSong.path)) {
    position.value = 0;
    duration.value = newSong.duration || 0;
    progress.value = 0;
    console.log('Props歌曲变化，进度条重置:', newSong.title);
  }
}, { deep: true });
</script>

<template>
  <div class="player-controls card">
    <div class="progress-bar">
      <div class="progress-info">
        <span class="time-display">{{ formatTime(position) }}</span>
        <span class="time-display">{{ formatTime(duration) }}</span>
      </div>
      <div class="progress-container" @click="handleProgressClick">
        <div class="progress" :style="{ width: `${progress}%` }" v-show="progress > 0"></div>
        <div 
          class="progress-handle" 
          :style="{ left: `${progress}%` }" 
          :class="{ active: progress > 0 || isPlaying, dragging: isDragging }"
          @mousedown="handleMouseDown"
          v-show="progress > 0"
        ></div>
      </div>
    </div>
    
    <div class="control-buttons">
      <button @click="handlePrevious" class="control-btn btn-secondary">
        <i class="icon-previous">⏮</i>
      </button>
      <button @click="handlePlayPause" class="control-btn play-pause-btn btn-primary" :class="{ 'playing': isPlaying }">
        <i v-if="!isPlaying" class="icon-play">▶</i>
        <i v-else class="icon-pause">⏸</i>
      </button>
      <button @click="handleNext" class="control-btn btn-secondary">
        <i class="icon-next">⏭</i>
      </button>
    </div>
    
    <!-- 保留原有的播放模式控制 -->
    <div class="play-mode-section">
      <PlayModeControl />
    </div>
  </div>
</template>

<style scoped>
.player-controls {
  padding: 1.25rem;
  position: relative;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 200px;
}

.player-controls::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  border-radius: var(--radius-lg);
  z-index: -1;
}

.progress-bar {
  margin-bottom: 1.25rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.time-display {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--background-glass);
  padding: 0.375rem 0.625rem;
  border-radius: var(--radius-sm);
  backdrop-filter: blur(10px);
}

.progress-container {
  height: 6px;
  background: rgba(102, 126, 234, 0.15);
  border-radius: var(--radius-sm);
  overflow: visible;
  cursor: pointer;
  position: relative;
  transition: all var(--transition-fast);
}

.progress-container:hover {
  height: 8px;
  background: rgba(102, 126, 234, 0.2);
}

.progress {
  height: 100%;
  background: var(--primary-gradient);
  transition: width 0.1s ease-out;
  border-radius: var(--radius-sm);
  position: relative;
  overflow: hidden;
}

.progress::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.progress-container:hover .progress {
  box-shadow: 0 0 12px rgba(102, 126, 234, 0.4);
}

.progress-handle {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  background: white;
  border: 3px solid var(--primary-color);
  border-radius: 50%;
  cursor: pointer;
  transition: all var(--transition-normal);
  opacity: 0;
  pointer-events: none;
  box-shadow: var(--shadow-md);
}

.progress-container:hover .progress-handle,
.progress-handle.active {
  opacity: 1;
  pointer-events: auto;
}

.progress-handle:hover {
  background: var(--primary-color);
  border-color: white;
  transform: translate(-50%, -50%) scale(1.2);
  box-shadow: var(--shadow-lg);
}

.progress-handle.dragging {
  cursor: grabbing;
  background: var(--primary-dark);
  border-color: white;
  transform: translate(-50%, -50%) scale(1.3);
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.5);
}

.control-buttons {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1.25rem;
}

.control-btn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.2rem;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
}

.control-btn i {
  position: relative;
  z-index: 2;
}

.play-pause-btn {
  width: 56px;
  height: 56px;
  font-size: 1.4rem;
  position: relative;
}

.play-pause-btn.playing {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: var(--shadow-primary);
  }
  50% {
    box-shadow: 0 6px 30px rgba(102, 126, 234, 0.5);
  }
}

.play-mode-section {
  display: flex;
  justify-content: center;
  padding-top: 1rem;
  border-top: 1px solid var(--border-light);
  margin-top: auto;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .player-controls {
    padding: 1rem;
    min-height: 180px;
  }
  
  .control-btn {
    width: 44px;
    height: 44px;
    font-size: 1.1rem;
  }
  
  .play-pause-btn {
    width: 52px;
    height: 52px;
    font-size: 1.3rem;
  }
  
  .control-buttons {
    gap: 0.75rem;
  }
  
  .time-display {
    font-size: 0.85rem;
    padding: 0.25rem 0.5rem;
  }
}
</style>
