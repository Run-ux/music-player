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

// 计算当前应该播放的视频文件路径
const currentVideoPath = computed(() => {
  if (!props.song) return '';
  
  console.log('计算视频路径:', {
    song: props.song.title,
    playbackMode: playerStore.currentPlaybackMode,
    mvPath: props.song.mvPath,
    mediaType: props.song.mediaType
  });
  
  // 如果当前播放模式是Video且歌曲有MV，则播放MV
  if (playerStore.currentPlaybackMode === MediaType.Video && props.song.mvPath) {
    console.log('使用MV路径:', props.song.mvPath);
    return props.song.mvPath;
  }
  
  // 如果歌曲本身就是视频文件，则播放歌曲本身
  if (props.song.mediaType === MediaType.Video) {
    console.log('使用视频文件路径:', props.song.path);
    return props.song.path;
  }
  
  console.log('无视频路径可用');
  return '';
});

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

// 监听当前视频路径变化
watch(currentVideoPath, async (newPath, oldPath) => {
  if (newPath && newPath !== oldPath) {
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
  
  // 优化：视频加载完成后立即开始播放（无论主播放器状态如何）
  if (videoElement.value) {
    videoElement.value.play().then(() => {
      isVideoPlaying.value = true;
      console.log('新视频自动开始播放');
      
      // 确保主播放器状态同步为播放
      if (!props.isPlaying) {
        playerStore.play();
      }
    }).catch((error) => {
      console.warn('视频自动播放失败，可能需要用户交互:', error);
      // 如果自动播放失败，但主播放器是播放状态，仍然尝试播放
      if (props.isPlaying) {
        setTimeout(() => {
          videoElement.value?.play().catch(console.error);
        }, 100);
      }
    });
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

// 处理视频时间更新 - 只更新前端，不触发后端
const handleTimeUpdate = () => {
  if (videoElement.value && isVideoLoaded.value) {
    const currentTime = Math.floor(videoElement.value.currentTime);
    const videoDuration = Math.floor(videoElement.value.duration);
    
    // 更新实际时长（如果还没设置）
    if (videoDuration > 0 && actualVideoDuration.value !== videoDuration) {
      actualVideoDuration.value = videoDuration;
    }
    
    // 关键修复：视频播放时只更新前端进度，绝不发送到后端
    if (!videoElement.value.paused && isVideoPlaying.value && !isUserSeeking.value) {
      // 只更新前端进度显示，完全不调用后端API
      playerStore.updateProgress(currentTime, videoDuration);
      // 移除：不再发送到后端，避免触发音频重置
      // sendProgressToBackend(currentTime, videoDuration);
    }
  }
};

// 添加日志控制变量
const lastLoggedTime = ref(-1);

// 添加跳转控制标志，避免循环触发
const isUserSeeking = ref(false);
const lastSeekPosition = ref(-1);
const isPlayerControlsJumping = ref(false); // 新增：主进度条跳转标志

// 监听主播放器的position变化来实现进度条跳转 - 完全重写
watch(() => playerStore.position, (newPosition, oldPosition) => {
  if (videoElement.value && isVideoLoaded.value && !isUserSeeking.value) {
    const currentVideoTime = Math.floor(videoElement.value.currentTime);
    
    // 更精确的跳转检测逻辑
    const positionDiff = Math.abs(newPosition - currentVideoTime);
    const isSignificantJump = positionDiff > 3; // 提高阈值到3秒
    const isNotNaturalProgress = Math.abs(newPosition - oldPosition) > 2;
    
    if (isSignificantJump && isNotNaturalProgress) {
      console.log('VideoPlayer: 检测到主进度条跳转，视频跳转到:', newPosition, '秒');
      
      // 设置跳转标志
      isUserSeeking.value = true;
      isPlayerControlsJumping.value = true;
      
      // 执行视频跳转
      videoElement.value.currentTime = newPosition;
      
      // 缩短重置时间，提高响应性
      setTimeout(() => {
        isUserSeeking.value = false;
        isPlayerControlsJumping.value = false;
        console.log('VideoPlayer: 跳转标志重置');
      }, 200);
    }
  }
});

// 新增：监听视频跳转事件，同步进度到主进度条
const handleVideoSeek = () => {
  // 只有在不是主进度条触发的跳转时才处理
  if (videoElement.value && actualVideoDuration.value > 0 && !isPlayerControlsJumping.value) {
    const currentTime = Math.floor(videoElement.value.currentTime);
    const duration = actualVideoDuration.value;
    
    console.log('VideoPlayer: 视频内置进度条跳转，同步到主进度条:', currentTime);
    
    // 关键修复：设置更长的保护标志，避免和主进度条冲突
    isUserSeeking.value = true;
    
    // 延迟一小段时间再更新，确保视频跳转完全完成
    setTimeout(() => {
      // 双重检查：确保视频确实跳转到了目标位置
      if (videoElement.value && Math.abs(videoElement.value.currentTime - currentTime) < 1) {
        // 只更新前端状态，不发送到后端，避免触发音频播放
        playerStore.updateProgress(currentTime, duration);
        console.log('VideoPlayer: 进度同步完成:', currentTime, '秒');
      } else {
        console.log('VideoPlayer: 跳转尚未完成，跳过进度同步');
      }
      
      // 延长保护时间，确保主进度条不会立即响应
      setTimeout(() => {
        isUserSeeking.value = false;
        console.log('VideoPlayer: 跳转保护解除');
      }, 300);
    }, 100);
  } else {
    console.log('VideoPlayer: 跳过进度条同步 - 可能是主进度条触发的跳转');
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
    
    // 关键修复：只更新前端进度和时长缓存，不发送到后端
    if (videoDuration > 0) {
      console.log('立即同步视频时长到前端显示:', videoDuration, '秒');
      playerStore.updateProgress(0, videoDuration);
      // 移除：不再发送到后端，避免触发音频重置
      // sendProgressToBackend(0, videoDuration);
      
      // 新增：更新PlayerStore中的视频时长缓存，让播放列表能显示正确时长
      playerStore.updateVideoDuration(props.song.path, videoDuration);
    }
    
    // 优化：元数据加载完成后，如果视频已经加载完成但还没开始播放，立即开始播放
    if (isVideoLoaded.value && !isVideoPlaying.value) {
      console.log('元数据加载完成，尝试开始播放视频');
      videoElement.value.play().then(() => {
        isVideoPlaying.value = true;
        console.log('视频在元数据加载后自动开始播放');
        
        // 确保主播放器状态同步为播放
        if (!props.isPlaying) {
          playerStore.play();
        }
      }).catch((error) => {
        console.warn('元数据加载后视频自动播放失败:', error);
      });
    }
  }
};

// 监听播放模式变化，确保模式切换时重新加载视频
watch(() => playerStore.currentPlaybackMode, async (newMode, oldMode) => {
  if (newMode !== oldMode) {
    console.log('播放模式变化:', oldMode, '->', newMode);
    
    // 如果切换到视频模式且有视频路径，重新加载视频
    if (newMode === MediaType.Video && currentVideoPath.value) {
      console.log('切换到视频模式，重新加载视频:', currentVideoPath.value);
      
      // 重置视频状态
      isVideoLoaded.value = false;
      loadingError.value = '';
      isVideoPlaying.value = false;
      
      const secureUrl = await getSecureVideoPath(currentVideoPath.value);
      if (secureUrl) {
        videoSrc.value = secureUrl;
        if (videoElement.value) {
          videoElement.value.load();
          
          // 关键修复：等待视频加载完成后立即开始播放
          videoElement.value.addEventListener('loadeddata', () => {
            if (props.isPlaying && videoElement.value) {
              videoElement.value.play().then(() => {
                isVideoPlaying.value = true;
                console.log('模式切换后视频自动开始播放');
              }).catch(console.error);
            }
          }, { once: true });
        }
      }
    } else if (newMode === MediaType.Audio) {
      console.log('切换到音频模式，清理视频资源');
      // 切换到音频模式时，完全停止视频播放并清理资源
      if (videoElement.value) {
        videoElement.value.pause();
        videoElement.value.currentTime = 0;
        videoElement.value.src = '';
      }
      videoSrc.value = '';
      isVideoLoaded.value = false;
      isVideoPlaying.value = false;
    }
  }
}, { immediate: true });

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
      <!-- 优化的视频元素 - 根据当前视频路径显示 -->
      <video
        v-if="currentVideoPath && videoSrc"
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
      <div v-if="!isVideoLoaded && currentVideoPath && !loadingError" class="video-loading">
        <div class="loading-spinner"></div>
        <p>正在加载视频...</p>
        <p class="video-info">{{ songTitle }}</p>
        <p class="debug-info">文件: {{ currentVideoPath }}</p>
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
      <!-- 显示当前播放模式 -->
      <div class="playback-mode-info">
        <span v-if="playerStore.currentPlaybackMode === MediaType.Video && props.song?.mvPath" class="mode-badge mv-mode">
          🎬 MV模式
        </span>
        <span v-else-if="props.song?.mediaType === MediaType.Video" class="mode-badge video-mode">
          📹 视频文件
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

/* 播放模式指示器样式 */
.playback-mode-info {
  margin-top: 0.5rem;
  display: flex;
  justify-content: center;
  gap: 0.5rem;
}

.mode-badge {
  padding: 0.2rem 0.6rem;
  border-radius: 12px;
  font-size: 0.9rem;
  color: white;
  background: rgba(255, 255, 255, 0.2);
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
}

.mv-mode {
  background: rgba(76, 175, 80, 0.2);
}

.video-mode {
  background: rgba(33, 150, 243, 0.2);
}
</style>