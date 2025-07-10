<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { SongInfo, MediaType } from '../stores/player';
import { usePlayerStore } from '../stores/player';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  song: SongInfo | null;
  isPlaying: boolean;
}>();

// 获取播放器store用于进度同步
const playerStore = usePlayerStore();

const videoElement = ref<HTMLVideoElement>();
const isVideoLoaded = ref(false);
const showControls = ref(false);
const controlsTimer = ref<number>();
const videoSrc = ref<string>('');
const loadingError = ref<string>('');

// 计算歌曲信息  
const songTitle = computed(() => {
  return props.song?.title || '未知视频';
});

const songArtist = computed(() => {
  return props.song?.artist || '';
});

// 计算视频缩略图
const videoThumbnail = computed(() => {
  if (props.song?.videoThumbnail) {
    return props.song.videoThumbnail;
  }
  return '';
});

// 获取安全的视频文件路径
const getSecureVideoPath = async (filePath: string) => {
  try {
    // 使用新的视频流方法
    const videoData = await invoke<number[]>('get_video_stream', { filePath });
    
    // 将数据转换为Blob
    const uint8Array = new Uint8Array(videoData);
    const blob = new Blob([uint8Array], { type: 'video/mp4' });
    const objectUrl = URL.createObjectURL(blob);
    
    console.log('生成video blob URL:', objectUrl);
    loadingError.value = '';
    return objectUrl;
  } catch (error) {
    console.error('获取视频流失败:', error);
    loadingError.value = `无法加载视频: ${error}`;
    return '';
  }
};

// 监听播放状态变化
watch(() => props.isPlaying, (isPlaying) => {
  if (videoElement.value && isVideoLoaded.value) {
    if (isPlaying) {
      videoElement.value.play().catch(console.error);
    } else {
      videoElement.value.pause();
    }
  }
});

// 监听歌曲变化
watch(() => props.song?.path, async (newPath, oldPath) => {
  if (newPath && newPath !== oldPath && props.song?.mediaType === MediaType.Video) {
    isVideoLoaded.value = false;
    loadingError.value = '';
    console.log('视频文件路径变化:', newPath);
    
    // 获取安全的视频路径
    const secureUrl = await getSecureVideoPath(newPath);
    if (secureUrl) {
      videoSrc.value = secureUrl;
      if (videoElement.value) {
        videoElement.value.load();
      }
    }
  }
}, { immediate: true });

// 处理视频加载完成
const handleVideoLoaded = () => {
  console.log('视频加载完成');
  isVideoLoaded.value = true;
  loadingError.value = '';
  if (props.isPlaying && videoElement.value) {
    videoElement.value.play().catch(console.error);
  }
};

// 处理视频加载错误
const handleVideoError = (event: Event) => {
  console.error('视频加载失败:', event);
  const target = event.target as HTMLVideoElement;
  loadingError.value = `视频加载失败: ${target.error?.message || '未知错误'}`;
  isVideoLoaded.value = false;
};

// 处理视频时间更新，同步到播放器store
const handleTimeUpdate = () => {
  if (videoElement.value && props.song?.duration) {
    const currentTime = Math.floor(videoElement.value.currentTime);
    const duration = props.song.duration;
    
    // 更新本地视频时间
    currentVideoTime.value = currentTime;
    
    // 更新播放器store的进度
    playerStore.updateProgress(currentTime, duration);
  }
};

// 处理来自播放器控制的跳转请求
const handleSeekFromControls = (targetTime: number) => {
  if (videoElement.value && isVideoLoaded.value) {
    console.log('视频跳转到:', targetTime, '秒');
    videoElement.value.currentTime = targetTime;
  }
};

// 监听播放器store的跳转事件
watch(() => playerStore.position, (newPosition, oldPosition) => {
  // 只有当位置变化超过1秒且不是正常播放进度时才执行跳转
  if (videoElement.value && isVideoLoaded.value && 
      Math.abs(newPosition - oldPosition) > 1 && 
      Math.abs(newPosition - videoElement.value.currentTime) > 1) {
    videoElement.value.currentTime = newPosition;
  }
});

// 处理视频播放结束
const handleVideoEnded = () => {
  // 自动播放下一首
  playerStore.next();
};

// 显示控制条
const showVideoControls = () => {
  showControls.value = true;
  
  // 清除之前的定时器
  if (controlsTimer.value) {
    clearTimeout(controlsTimer.value);
  }
  
  // 3秒后隐藏控制条
  controlsTimer.value = setTimeout(() => {
    showControls.value = false;
  }, 3000);
};

// 隐藏控制条
const hideVideoControls = () => {
  if (controlsTimer.value) {
    clearTimeout(controlsTimer.value);
  }
  showControls.value = false;
};

// 视频进度相关
const currentVideoTime = ref(0);
const videoProgress = computed(() => {
  if (!props.song?.duration) return 0;
  return (currentVideoTime.value / props.song.duration) * 100;
});

// 格式化视频时间
const formatVideoTime = (timeInSeconds: number) => {
  const minutes = Math.floor(timeInSeconds / 60);
  const seconds = Math.floor(timeInSeconds % 60);
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
};

// 处理视频进度条点击
const handleVideoProgressClick = (event: MouseEvent) => {
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const offsetX = event.clientX - rect.left;
  const width = rect.width;
  const clickPercent = offsetX / width;
  
  if (props.song?.duration) {
    const targetTime = clickPercent * props.song.duration;
    handleSeekFromControls(targetTime);
  }
};

// 处理进度条拖动
let isDragging = false;

const handleVideoProgressMouseDown = (event: MouseEvent) => {
  isDragging = true;
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const width = rect.width;
  
  const onMouseMove = (moveEvent: MouseEvent) => {
    if (!isDragging) return;
    const moveX = moveEvent.clientX - rect.left;
    const movePercent = moveX / width;
    
    if (props.song?.duration) {
      const targetTime = movePercent * props.song.duration;
      handleSeekFromControls(targetTime);
    }
  };
  
  const onMouseUp = () => {
    isDragging = false;
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
  };
  
  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
};

onMounted(() => {
  // 初始化时显示控制条
  showVideoControls();
});

onUnmounted(() => {
  if (controlsTimer.value) {
    clearTimeout(controlsTimer.value);
  }
});
</script>

<template>
  <div class="video-player">
    <div 
      class="video-container"
      @mousemove="showVideoControls"
      @mouseleave="hideVideoControls"
    >
      <!-- 视频元素 -->
      <video
        v-if="props.song?.mediaType === MediaType.Video && videoSrc"
        ref="videoElement"
        class="video-element"
        :src="videoSrc"
        :poster="videoThumbnail"
        @loadeddata="handleVideoLoaded"
        @error="handleVideoError"
        @timeupdate="handleTimeUpdate"
        @ended="handleVideoEnded"
        preload="metadata"
        controls
      />
      
      <!-- 视频加载中状态 -->
      <div v-if="!isVideoLoaded && props.song?.mediaType === MediaType.Video && !loadingError" class="video-loading">
        <div class="loading-spinner"></div>
        <p>正在加载视频...</p>
        <p class="debug-info">文件: {{ props.song?.path }}</p>
        <p class="debug-info">安全路径: {{ videoSrc }}</p>
      </div>
      
      <!-- 错误状态 -->
      <div v-if="loadingError" class="video-error">
        <div class="error-icon">⚠️</div>
        <p class="error-message">{{ loadingError }}</p>
        <p class="debug-info">原始路径: {{ props.song?.path }}</p>
        <p class="debug-info">转换路径: {{ videoSrc }}</p>
      </div>
      
      <!-- 视频控制条 -->
      <div 
        v-show="showControls && isVideoLoaded" 
        class="video-controls"
        @mouseenter="showVideoControls"
      >
        <!-- 视频专用进度条 -->
        <div class="video-progress-bar" v-if="props.song?.duration">
          <div class="video-progress-container" @click="handleVideoProgressClick">
            <div class="video-progress" :style="{ width: `${videoProgress}%` }"></div>
            <div 
              class="video-progress-handle" 
              :style="{ left: `${videoProgress}%` }"
              @mousedown="handleVideoProgressMouseDown"
            ></div>
          </div>
          <div class="video-time-info">
            <span>{{ formatVideoTime(currentVideoTime) }}</span>
            <span>{{ formatVideoTime(props.song.duration) }}</span>
          </div>
        </div>
        
        <div class="video-info">
          <div class="video-title">{{ songTitle }}</div>
          <div v-if="songArtist" class="video-artist">{{ songArtist }}</div>
        </div>
      </div>
    </div>
    
    <!-- 视频信息显示 -->
    <div class="video-details">
      <div class="video-title-main">{{ songTitle }}</div>
      <div v-if="songArtist" class="video-artist-main">{{ songArtist }}</div>
    </div>
  </div>
</template>

<style scoped>
.video-player {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #000;
  border-radius: 8px;
  overflow: hidden;
}

.video-container {
  position: relative;
  width: 100%;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  cursor: none;
}

.video-element {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.video-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
}

.video-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  max-width: 80%;
}

.error-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.error-message {
  font-size: 1.1rem;
  margin-bottom: 1rem;
  color: #ff6b6b;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top: 3px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 10px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.video-controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
  padding: 20px;
  transition: opacity 0.3s ease;
}

.video-progress-bar {
  width: 100%;
  margin-bottom: 10px;
}

.video-progress-container {
  position: relative;
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  cursor: pointer;
}

.video-progress {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: #ff6b6b;
  border-radius: 2px;
}

.video-progress-handle {
  position: absolute;
  top: -8px;
  width: 16px;
  height: 16px;
  background: #ff6b6b;
  border-radius: 50%;
  cursor: pointer;
  margin-left: -8px;
  transition: background 0.3s;
}

.video-progress-handle:hover {
  background: #ff4c4c;
}

.video-time-info {
  display: flex;
  justify-content: space-between;
  color: white;
  font-size: 0.9rem;
  margin-top: 4px;
}

.video-info {
  color: white;
}

.video-title {
  font-size: 1.2rem;
  font-weight: 600;
  margin-bottom: 4px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
}

.video-artist {
  font-size: 1rem;
  opacity: 0.9;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
}

.video-details {
  padding: 1rem;
  background: #f9f9f9;
  text-align: center;
}

.video-title-main {
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #333;
}

.video-artist-main {
  font-size: 0.9rem;
  color: #666;
}

.debug-info {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.7);
  margin: 0.2rem 0;
  word-break: break-all;
}

/* 鼠标悬停时显示指针 */
.video-container:hover {
  cursor: default;
}
</style>