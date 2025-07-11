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

// 检查当前歌曲是否有MV
const hasMv = computed(() => {
  return props.song?.mvPath !== undefined && props.song?.mvPath !== null;
});

// 当前播放模式
const isVideoMode = computed(() => {
  return playerStore.currentPlaybackMode === MediaType.Video;
});

// 切换播放模式
const togglePlaybackMode = async () => {
  if (!hasMv.value) {
    console.warn('当前歌曲没有MV，无法切换模式');
    return;
  }
  
  try {
    const newMode = isVideoMode.value ? MediaType.Audio : MediaType.Video;
    console.log('切换播放模式:', isVideoMode.value ? 'MV -> 音频' : '音频 -> MV');
    
    // 调用后端切换播放模式
    await playerStore.setPlaybackMode(newMode);
    
    console.log('播放模式切换成功:', newMode);
  } catch (error) {
    console.error('切换播放模式失败:', error);
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
      
      <!-- MV切换按钮 -->
      <div v-if="hasMv" class="mv-toggle-container">
        <button 
          @click="togglePlaybackMode"
          class="mv-toggle-btn"
          :class="{ 'video-mode': isVideoMode }"
          :title="isVideoMode ? '切换到音频模式' : '切换到MV模式'"
        >
          <svg v-if="!isVideoMode" class="icon" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z"/>
          </svg>
          <svg v-else class="icon" viewBox="0 0 24 24">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z"/>
          </svg>
          <span class="mode-text">{{ isVideoMode ? 'MV' : '音频' }}</span>
        </button>
      </div>
    </div>
    
    <div class="song-details">
      <div class="song-title">{{ songTitle }}</div>
      <div class="song-artist">{{ songArtist }}</div>
      <div class="song-album">{{ songAlbum }}</div>
      
      <!-- 显示播放模式提示 -->
      <div v-if="hasMv" class="playback-mode-indicator">
        <span class="mode-indicator" :class="{ 'video-mode': isVideoMode }">
          {{ isVideoMode ? '🎬 MV模式' : '🎵 音频模式' }}
        </span>
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

.mv-toggle-container {
  position: absolute;
  bottom: -10px;
  right: -10px;
}

.mv-toggle-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  background: #fff;
  border: 2px solid #ddd;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  font-size: 12px;
  font-weight: 500;
}

.mv-toggle-btn:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.mv-toggle-btn.video-mode {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-color: #667eea;
}

.mv-toggle-btn .icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.mode-text {
  font-weight: 600;
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
  margin-bottom: 0.5rem;
}

.playback-mode-indicator {
  margin-top: 0.5rem;
}

.mode-indicator {
  display: inline-block;
  padding: 4px 8px;
  background: #f0f0f0;
  border-radius: 12px;
  font-size: 0.8rem;
  font-weight: 500;
  color: #666;
  transition: all 0.3s ease;
}

.mode-indicator.video-mode {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}
</style>
