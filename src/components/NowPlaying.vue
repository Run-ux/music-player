<script setup lang="ts">
import { computed } from 'vue';

import { SongInfo } from '../stores/player';

const props = defineProps<{
  song: SongInfo | null;
  isPlaying: boolean;
}>();

// 计算专辑封面
const albumCover = computed(() => {
  // 这里应该根据实际情况返回专辑封面，目前使用默认图片
  return props.song?.albumCover || '/default-album.png';
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
</script>

<template>
  <div class="now-playing">
    <div class="album-cover">
      <div class="cover-container" :class="{ rotating: isPlaying }">
        <img 
          :src="albumCover" 
          alt="Album Cover" 
          class="cover-image"
          @error="($event.target as HTMLImageElement).src = '/default-album.png'"
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
}

.cover-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: all 0.3s;
}

.rotating {
  animation: rotate 20s linear infinite;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
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
