<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import PlayModeControl from './PlayModeControl.vue';
import { usePlayerStore, SongInfo, MediaType } from '../stores/player';

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
  if (props.isPlaying) {
    emit('pause');
  } else {
    emit('play');
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
  // 关键修复：增加更智能的保护逻辑
  const shouldUpdateProgress = !isDragging.value && !isUserJumping.value;
  
  if (shouldUpdateProgress) {
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

// 监听当前歌曲变化，确保props变化时也重置进度条
watch(() => props.currentSong, (newSong, oldSong) => {
  if (newSong && (!oldSong || newSong.path !== oldSong.path)) {
    position.value = 0;
    duration.value = newSong.duration || 0;
    progress.value = 0;
    console.log('Props歌曲变化，进度条重置:', newSong.title);
  }
}, { deep: true });

// 播放模式相关状态和计算属性
const supportsModeSwitch = computed(() => {
  if (!props.currentSong) return false;
  
  // 纯视频文件不支持切换到音频模式
  if (props.currentSong.mediaType === MediaType.Video) {
    return false;
  }
  
  // 音频文件只有在有MV时才支持切换
  return props.currentSong.mediaType === MediaType.Audio && props.currentSong.mvPath;
});

// 当前播放模式
const isVideoMode = computed(() => {
  return playerStore.currentPlaybackMode === MediaType.Video;
});

// 切换播放模式
const handleTogglePlaybackMode = async () => {
  if (!supportsModeSwitch.value) {
    console.warn('当前歌曲不支持播放模式切换');
    return;
  }
  
  try {
    const oldMode = isVideoMode.value ? MediaType.Video : MediaType.Audio;
    const newMode = isVideoMode.value ? MediaType.Audio : MediaType.Video;
    console.log('🔄 PlayerControls切换播放模式:', oldMode, '->', newMode);
    
    // 调用后端切换播放模式
    await playerStore.setPlaybackMode(newMode);
    
    // 关键修复：视频切音频后给一个短暂延迟确保后端处理完成
    if (oldMode === MediaType.Video && newMode === MediaType.Audio) {
      console.log('视频切音频，等待后端完成处理...');
      
      // 等待一小段时间确保后端音频播放器准备就绪
      await new Promise(resolve => setTimeout(resolve, 300));
      
      // 检查播放状态，如果不是播放状态则强制播放
      if (!props.isPlaying) {
        console.log('检测到音频未自动播放，手动启动播放');
        try {
          await playerStore.play();
        } catch (error) {
          console.warn('手动启动音频播放失败:', error);
        }
      }
    }
    
    console.log('✅ 播放模式切换成功:', newMode);
  } catch (error) {
    console.error('❌ 切换播放模式失败:', error);
  }
};
</script>

<template>
  <div class="player-controls">
    <div class="progress-bar">
      <div class="progress-info">
        <span>{{ formatTime(position) }}</span>
        <span>{{ formatTime(duration) }}</span>
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
      <button @click="handlePrevious" class="control-btn">
        <i class="icon-previous">⏮</i>
      </button>
      <button @click="handlePlayPause" class="control-btn play-pause-btn" :class="{ 'play': !isPlaying, 'pause': isPlaying }">
        <i v-if="!isPlaying" class="icon-play">▶</i>
        <i v-else class="icon-pause">⏸</i>
      </button>
      <button @click="handleNext" class="control-btn">
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
  height: 8px;
  background-color: #ddd;
  border-radius: 4px;
  overflow: visible; /* 改为visible以便手柄可以正确显示 */
  cursor: pointer;
  position: relative;
  transition: height 0.2s ease;
}

.progress-container:hover {
  height: 10px;
}

.progress {
  height: 100%;
  background: linear-gradient(90deg, #4caf50, #66bb6a);
  transition: width 0.1s ease-out;
  border-radius: 4px;
  min-width: 0; /* 确保宽度为0时不显示 */
}

.progress-container:hover .progress {
  box-shadow: 0 0 8px rgba(76, 175, 80, 0.4);
}

.progress-handle {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 12px; /* 从14px减小到12px */
  height: 12px; /* 从14px减小到12px */
  background: #4caf50;
  border: 2px solid #fff;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s;
  opacity: 0;
  pointer-events: none;
}

/* 只有在有进度或者hover时才显示手柄 */
.progress-container:hover .progress-handle,
.progress-handle.active {
  opacity: 1;
  pointer-events: auto;
}

.progress-handle:hover {
  background: #45a049;
  transform: translate(-50%, -50%) scale(1.15); /* 缩小hover放大倍数 */
}

/* 添加拖拽状态样式 */
.progress-handle.dragging {
  cursor: grabbing;
  background: #388e3c;
  transform: translate(-50%, -50%) scale(1.2);
}

.control-buttons {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
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

.play-mode-section {
  display: flex;
  justify-content: center;
  padding-top: 0.5rem;
  border-top: 1px solid #eee;
}
</style>
