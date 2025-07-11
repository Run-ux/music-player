<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { SongInfo, MediaType, usePlayerStore } from '../stores/player';

const props = defineProps<{
  song: SongInfo | null;
  isPlaying: boolean;
}>();

const playerStore = usePlayerStore();

// 封面旋转相关状态
const coverElement = ref<HTMLElement>();
const animationId = ref<number>();
const currentRotation = ref(0);
const ROTATION_SPEED = 18; // 度/秒 (20秒一圈)

// 计算专辑封面
const albumCover = computed(() => {
  if (props.song?.albumCover) {
    return props.song.albumCover;
  } else {
    return '/src/assets/default-cover.jpg';
  }
});

// 计算歌曲信息  
const songTitle = computed(() => {
  return props.song?.title || '未知歌曲';
});

const songArtist = computed(() => {
  return props.song?.artist || '未知艺术家';
});

const songAlbum = computed(() => {
  return props.song?.album || '未知专辑';
});

// 播放模式切换相关逻辑
const supportsModeSwitch = computed(() => {
  if (!props.song) return false;
  
  // 纯视频文件不支持切换到音频模式
  if (props.song.mediaType === MediaType.Video) {
    return false;
  }
  
  // 音频文件只有在有MV时才支持切换
  return props.song.mediaType === MediaType.Audio && props.song.mvPath;
});

// 当前播放模式
const isVideoMode = computed(() => {
  return playerStore.currentPlaybackMode === MediaType.Video;
});

// 切换播放模式
const togglePlaybackMode = async () => {
  if (!supportsModeSwitch.value) {
    console.warn('当前歌曲不支持播放模式切换');
    return;
  }
  
  try {
    const oldMode = isVideoMode.value ? MediaType.Video : MediaType.Audio;
    const newMode = isVideoMode.value ? MediaType.Audio : MediaType.Video;
    console.log('🔄 NowPlaying切换播放模式:', oldMode, '->', newMode);
    
    // 调用后端切换播放模式
    await playerStore.setPlaybackMode(newMode);
    
    // 关键修复：视频切音频后给一个短暂延迟确保后端处理完成
    if (oldMode === MediaType.Video && newMode === MediaType.Audio) {
      console.log('视频切音频，等待后端完成处理...');
      
      // 等待一小段时间确保后端音频播放器准备就绪
      await new Promise(resolve => setTimeout(resolve, 300));
      
      // 检查播放状态，如果不是播放状态则强制播放
      if (!playerStore.isPlaying) {
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

// 简化封面旋转控制逻辑 - 只要是播放状态就旋转
const shouldRotate = computed(() => {
  return props.isPlaying;
});

// 旋转动画
const animate = () => {
  if (shouldRotate.value) {
    currentRotation.value += ROTATION_SPEED / 60; // 每帧增加的角度
    
    if (currentRotation.value >= 360) {
      currentRotation.value -= 360;
    }
  }
  
  if (coverElement.value) {
    coverElement.value.style.transform = `rotate(${currentRotation.value}deg)`;
  }
  
  animationId.value = requestAnimationFrame(animate);
};

// 监听歌曲变化，重置旋转角度
watch(() => props.song?.path, (newPath, oldPath) => {
  if (newPath && newPath !== oldPath) {
    currentRotation.value = 0;
    if (coverElement.value) {
      coverElement.value.style.transform = 'rotate(0deg)';
    }
  }
}, { immediate: true });

onMounted(() => {
  animationId.value = requestAnimationFrame(animate);
});

onUnmounted(() => {
  if (animationId.value) {
    cancelAnimationFrame(animationId.value);
  }
});
</script>

<template>
  <div class="now-playing">
    <div class="album-cover">
      <div 
        ref="coverElement"
        class="cover-container"
      >
        <img 
          :src="albumCover" 
          alt="Album Cover" 
          class="cover-image"
          @error="($event.target as HTMLImageElement).src = '/src/assets/default-cover.jpg'"
        />
      </div>
    </div>
    
    <div class="song-details">
      <div class="song-title">{{ songTitle }}</div>
      <div class="song-artist">{{ songArtist }}</div>
      <div class="song-album">{{ songAlbum }}</div>
      
      <!-- 播放模式切换按钮 - 横向排列在歌曲信息下方 -->
      <div v-if="supportsModeSwitch" class="mode-switch-controls">
        <button 
          @click="togglePlaybackMode"
          class="mode-switch-btn"
          :class="{ 'active': !isVideoMode }"
          :title="'音频模式'"
        >
          <svg class="mode-icon" viewBox="0 0 24 24">
            <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
          </svg>
          <span class="mode-text">音频</span>
        </button>
        
        <button 
          @click="togglePlaybackMode"
          class="mode-switch-btn"
          :class="{ 'active': isVideoMode }"
          :title="'MV模式'"
        >
          <svg class="mode-icon" viewBox="0 0 24 24">
            <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
          </svg>
          <span class="mode-text">MV</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.now-playing {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.5rem;
  background: #f9f9f9;
  border-radius: 8px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
}

.album-cover {
  width: 100%;
  max-width: 300px;
  margin-bottom: 1.5rem;
  position: relative;
}

.cover-container {
  width: 100%;
  padding-top: 100%; /* 1:1 Aspect Ratio */
  position: relative;
  border-radius: 50%;
  overflow: hidden;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  transform-origin: center center;
}

.cover-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.song-details {
  text-align: center;
  width: 100%;
}

.song-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #333;
}

.song-artist {
  font-size: 1.1rem;
  color: #555;
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-album {
  font-size: 0.9rem;
  color: #777;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 1.25rem;
}

/* 播放模式切换按钮样式 - 横向排列 */
.mode-switch-controls {
  display: flex;
  justify-content: center;
  gap: 0.75rem;
  margin-top: 1rem;
  width: 100%;
}

.mode-switch-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: rgba(255, 255, 255, 0.8);
  border: 2px solid rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  backdrop-filter: blur(5px);
  min-width: 80px;
  color: #666;
}

.mode-switch-btn:hover {
  transform: translateY(-2px);
  background: rgba(255, 255, 255, 0.95);
  border-color: rgba(102, 126, 234, 0.3);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
}

.mode-switch-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
}

.mode-switch-btn.active:hover {
  transform: translateY(-2px);
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

.mode-icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
  transition: all 0.3s ease;
}

.mode-switch-btn:hover .mode-icon {
  transform: scale(1.1);
}

.mode-text {
  font-size: 0.85rem;
  font-weight: 600;
  white-space: nowrap;
}

/* 响应式设计 */
@media (max-width: 480px) {
  .song-title {
    font-size: 1.3rem;
  }
  
  .mode-switch-controls {
    gap: 0.5rem;
  }
  
  .mode-switch-btn {
    padding: 0.625rem 0.75rem;
    min-width: 70px;
  }
  
  .mode-icon {
    width: 20px;
    height: 20px;
  }
  
  .mode-text {
    font-size: 0.8rem;
  }
}

/* 触摸设备优化 */
@media (hover: none) and (pointer: coarse) {
  .mode-switch-btn {
    padding: 0.875rem 1rem;
  }
  
  .mode-switch-btn:hover {
    transform: none;
  }
  
  .mode-switch-btn:active {
    transform: scale(0.98);
    transition: transform 0.1s ease;
  }
}
</style>
