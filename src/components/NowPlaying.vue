<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { SongInfo } from '../stores/player';

const props = defineProps<{
  song: SongInfo | null;
  isPlaying: boolean;
}>();

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
}
</style>
