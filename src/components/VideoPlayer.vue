<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { SongInfo, MediaType } from '../stores/player';
import { usePlayerStore } from '../stores/player';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

const props = defineProps<{
  song: SongInfo | null;
  isPlaying: boolean;
}>();

// 获取播放器store用于进度同步
const playerStore = usePlayerStore();

const videoElement = ref<HTMLVideoElement>();
const isVideoLoaded = ref(false);
const videoSrc = ref<string>('');
const loadingError = ref<string>('');
const isVideoPlaying = ref(false);

// 添加实际视频时长状态
const actualVideoDuration = ref<number>(0);

// 计算显示的时长 - 优先使用视频实际时长
const displayDuration = computed(() => {
  if (actualVideoDuration.value > 0) {
    return actualVideoDuration.value;
  }
  return props.song?.duration || 0;
});

// 格式化时长显示
const formatDuration = (seconds: number) => {
  if (seconds <= 0) return '--:--';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${String(secs).padStart(2, '0')}`;
};

// 计算歌曲信息  
const songTitle = computed(() => {
  return props.song?.title || '未知视频';
});

const songArtist = computed(() => {
  return props.song?.artist || '';
});

// 获取安全的视频文件路径 - 使用Tauri的convertFileSrc API
const getSecureVideoPath = async (filePath: string) => {
  try {
    console.log('原始视频文件路径:', filePath);
    
    // 使用Tauri的convertFileSrc来转换文件路径
    const convertedUrl = convertFileSrc(filePath);
    console.log('转换后的视频URL:', convertedUrl);
    
    loadingError.value = '';
    return convertedUrl;
  } catch (error) {
    console.error('转换视频路径失败:', error);
    loadingError.value = `无法转换视频路径: ${error}`;
    return '';
  }
};

// 监听播放状态变化 - 与主播放器完全同步
watch(() => props.isPlaying, async (isPlaying) => {
  if (videoElement.value && isVideoLoaded.value) {
    try {
      if (isPlaying && !isVideoPlaying.value) {
        console.log('主播放器控制：开始播放视频');
        await videoElement.value.play();
        isVideoPlaying.value = true;
      } else if (!isPlaying && isVideoPlaying.value) {
        console.log('主播放器控制：暂停视频');
        videoElement.value.pause();
        isVideoPlaying.value = false;
      }
    } catch (error) {
      console.error('视频播放控制失败:', error);
    }
  }
});

// 监听歌曲变化
watch(() => props.song?.path, async (newPath, oldPath) => {
  if (newPath && newPath !== oldPath && props.song?.mediaType === MediaType.Video) {
    isVideoLoaded.value = false;
    loadingError.value = '';
    isVideoPlaying.value = false;
    console.log('切换视频文件:', newPath);
    
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
  console.log('视频加载完成，可以播放');
  isVideoLoaded.value = true;
  loadingError.value = '';
  
  // 如果主播放器处于播放状态，自动开始播放视频
  if (props.isPlaying && videoElement.value) {
    videoElement.value.play().then(() => {
      isVideoPlaying.value = true;
      console.log('视频自动开始播放');
    }).catch(console.error);
  }
};

// 处理视频加载错误
const handleVideoError = (event: Event) => {
  console.error('视频加载失败:', event);
  const target = event.target as HTMLVideoElement;
  const errorMessage = target.error?.message || '未知错误';
  loadingError.value = `视频加载失败: ${errorMessage}`;
  isVideoLoaded.value = false;
  isVideoPlaying.value = false;
};

// 新增：直接向后端发送进度更新的函数
const sendProgressToBackend = async (position: number, duration: number) => {
  try {
    // 通过 invoke 调用后端接口发送进度更新
    await invoke('update_video_progress', { 
      position: Math.floor(position), 
      duration: Math.floor(duration) 
    });
  } catch (error) {
    console.error('发送视频进度失败:', error);
  }
};

// 处理视频时间更新 - 同步到主播放器进度
const handleTimeUpdate = () => {
  if (videoElement.value && isVideoLoaded.value) {
    const currentTime = Math.floor(videoElement.value.currentTime);
    const videoDuration = Math.floor(videoElement.value.duration);
    
    // 更新实际时长（如果还没设置）
    if (videoDuration > 0 && actualVideoDuration.value !== videoDuration) {
      actualVideoDuration.value = videoDuration;
    }
    
    // 只有在视频真正播放时才更新进度（避免拖拽干扰）
    if (!videoElement.value.paused && isVideoPlaying.value) {
      // 同步到主播放器进度，使用实际视频时长
      playerStore.updateProgress(currentTime, videoDuration);
      // 同时发送到后端，确保后端状态同步
      sendProgressToBackend(currentTime, videoDuration);
      
      // 减少日志输出频率，只在整秒变化时输出
      if (currentTime !== lastLoggedTime.value) {
        console.log('视频进度同步:', currentTime, '/', videoDuration);
        lastLoggedTime.value = currentTime;
      }
    }
  }
};

// 添加日志控制变量
const lastLoggedTime = ref(-1);

// 添加跳转控制标志，避免循环触发
const isUserSeeking = ref(false);
const lastSeekPosition = ref(-1);

// 监听主播放器的position变化来实现进度条跳转
watch(() => playerStore.position, (newPosition, oldPosition) => {
  if (videoElement.value && isVideoLoaded.value && !isUserSeeking.value) {
    const currentVideoTime = Math.floor(videoElement.value.currentTime);
    
    // 检测是否是用户主动跳转（位置差异大且不是自然播放进度）
    const positionDiff = Math.abs(newPosition - currentVideoTime);
    const isSignificantJump = positionDiff > 2;
    const isNotNaturalProgress = Math.abs(newPosition - oldPosition) > 2;
    
    if (isSignificantJump && isNotNaturalProgress) {
      console.log('检测到进度条跳转，视频跳转到:', newPosition, '秒');
      isUserSeeking.value = true;
      lastSeekPosition.value = newPosition;
      videoElement.value.currentTime = newPosition;
      
      // 延迟重置标志
      setTimeout(() => {
        isUserSeeking.value = false;
      }, 200);
    }
  }
});

// 新增：监听视频跳转事件，同步进度
const handleVideoSeek = () => {
  if (videoElement.value && actualVideoDuration.value > 0 && !isUserSeeking.value) {
    const currentTime = Math.floor(videoElement.value.currentTime);
    const duration = actualVideoDuration.value;
    
    console.log('视频手动跳转，同步进度:', currentTime);
    
    // 设置标志避免循环触发
    isUserSeeking.value = true;
    playerStore.updateProgress(currentTime, duration);
    sendProgressToBackend(currentTime, duration);
    
    setTimeout(() => {
      isUserSeeking.value = false;
    }, 100);
  }
};

// 处理视频播放/暂停状态变化
const handleVideoPlay = () => {
  console.log('视频开始播放');
  isVideoPlaying.value = true;
  // 如果主播放器不是播放状态，同步更新
  if (!props.isPlaying) {
    playerStore.play();
  }
};

const handleVideoPause = () => {
  console.log('视频暂停');
  isVideoPlaying.value = false;
  // 如果主播放器是播放状态，同步更新
  if (props.isPlaying) {
    playerStore.pause();
  }
};

// 处理视频播放结束
const handleVideoEnded = () => {
  console.log('视频播放结束，切换下一首');
  isVideoPlaying.value = false;
  playerStore.next();
};

// 处理视频元数据加载完成
const handleVideoLoadedMetadata = () => {
  if (videoElement.value && props.song) {
    const videoDuration = Math.floor(videoElement.value.duration);
    console.log('视频元数据加载完成，时长:', videoDuration, '秒');
    
    // 更新实际视频时长
    actualVideoDuration.value = videoDuration;
    
    // 立即同步正确的时长到播放器系统，确保进度条使用正确的时长
    if (videoDuration > 0) {
      console.log('立即同步视频时长到播放器系统:', videoDuration, '秒');
      playerStore.updateProgress(0, videoDuration);
      sendProgressToBackend(0, videoDuration);
      
      // 新增：更新PlayerStore中的视频时长缓存，让播放列表能显示正确时长
      playerStore.updateVideoDuration(props.song.path, videoDuration);
    }
  }
};

onMounted(() => {
  console.log('VideoPlayer组件挂载完成');
});

onUnmounted(() => {
  console.log('VideoPlayer组件卸载');
});
</script>

<template>
  <div class="video-player">
    <div class="video-container">
      <!-- 优化的视频元素 - 启用所有原生控制功能 -->
      <video
        v-if="props.song?.mediaType === MediaType.Video && videoSrc"
        ref="videoElement"
        class="video-element"
        :src="videoSrc"
        @loadeddata="handleVideoLoaded"
        @loadedmetadata="handleVideoLoadedMetadata"
        @error="handleVideoError"
        @timeupdate="handleTimeUpdate"
        @seeked="handleVideoSeek"
        @play="handleVideoPlay"
        @pause="handleVideoPause"
        @ended="handleVideoEnded"
        preload="metadata"
        controls
        controlsList=""
        disablePictureInPicture="false"
      >
        您的浏览器不支持视频播放。
      </video>
      
      <!-- 视频加载中状态 -->
      <div v-if="!isVideoLoaded && props.song?.mediaType === MediaType.Video && !loadingError" class="video-loading">
        <div class="loading-spinner"></div>
        <p>正在加载视频...</p>
        <p class="video-info">{{ songTitle }}</p>
        <p class="debug-info">文件: {{ props.song?.path }}</p>
      </div>
      
      <!-- 错误状态 -->
      <div v-if="loadingError" class="video-error">
        <div class="error-icon">⚠️</div>
        <p class="error-message">{{ loadingError }}</p>
        <p class="video-info">{{ songTitle }}</p>
        <button @click="() => videoSrc && handleVideoLoaded()" class="retry-button">
          重试
        </button>
      </div>
    </div>
    
    <!-- 视频信息显示 -->
    <div class="video-details">
      <div class="video-title-main">{{ songTitle }}</div>
      <div v-if="songArtist" class="video-artist-main">{{ songArtist }}</div>
      <div class="video-status">
        <span class="status-indicator" :class="{ playing: isVideoPlaying }">
          {{ isVideoPlaying ? '播放中' : '已暂停' }}
        </span>
        <span v-if="displayDuration > 0" class="duration-info">
          时长: {{ formatDuration(displayDuration) }}
        </span>
      </div>
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
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.video-container {
  position: relative;
  width: 100%;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  min-height: 300px;
}

.video-element {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
}

/* 确保视频控制条可见且功能完整 */
.video-element::-webkit-media-controls-panel {
  background-color: rgba(0, 0, 0, 0.8);
}

.video-element::-webkit-media-controls-play-button,
.video-element::-webkit-media-controls-volume-slider,
.video-element::-webkit-media-controls-timeline,
.video-element::-webkit-media-controls-current-time-display,
.video-element::-webkit-media-controls-time-remaining-display,
.video-element::-webkit-media-controls-fullscreen-button {
  color: white;
  opacity: 1;
}

.video-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  background: rgba(0, 0, 0, 0.8);
  padding: 2rem;
  border-radius: 8px;
  backdrop-filter: blur(10px);
}

.video-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  background: rgba(0, 0, 0, 0.9);
  padding: 2rem;
  border-radius: 8px;
  max-width: 80%;
  backdrop-filter: blur(10px);
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

.retry-button {
  background: #4caf50;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  margin-top: 1rem;
  transition: background 0.3s;
}

.retry-button:hover {
  background: #45a049;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top: 3px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.video-details {
  padding: 1rem;
  background: linear-gradient(135deg, #f9f9f9 0%, #e8e8e8 100%);
  text-align: center;
  border-top: 1px solid #ddd;
}

.video-title-main {
  font-size: 1.2rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #333;
  text-shadow: 0 1px 2px rgba(255, 255, 255, 0.8);
}

.video-artist-main {
  font-size: 1rem;
  color: #666;
  margin-bottom: 0.5rem;
}

.video-status {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  font-size: 0.9rem;
  color: #777;
}

.status-indicator {
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  background: #e0e0e0;
  transition: all 0.3s;
}

.status-indicator.playing {
  background: #4caf50;
  color: white;
}

.duration-info {
  font-family: monospace;
}

.video-info {
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.debug-info {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.7);
  margin: 0.2rem 0;
  word-break: break-all;
}
</style>