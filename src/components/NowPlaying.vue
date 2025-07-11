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
  
  // 总是显示音频按钮，只要有歌曲就显示
  // 如果是纯视频文件，只显示MV按钮
  // 如果是音频文件，总是显示音频按钮，有MV时还显示MV按钮
  return true;
});

// 显示音频按钮的条件
const showAudioButton = computed(() => {
  if (!props.song) return false;
  // 纯视频文件不显示音频按钮
  return props.song.mediaType !== MediaType.Video;
});

// 显示MV按钮的条件
const showVideoButton = computed(() => {
  if (!props.song) return false;
  // 纯视频文件总是显示MV按钮
  if (props.song.mediaType === MediaType.Video) return true;
  // 音频文件有MV时显示MV按钮
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

//只要是播放状态就旋转
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
  <div class="now-playing card">
    <div class="album-cover animate-scale-in">
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
    
    <div class="song-details animate-slide-up-stagger">
      <div class="song-title">{{ songTitle }}</div>
      <div class="song-artist">{{ songArtist }}</div>
      <div class="song-album">{{ songAlbum }}</div>
      
      <!-- 播放模式切换按钮 -->
      <div v-if="supportsModeSwitch" class="mode-switch-controls">
        <button 
          v-if="showAudioButton"
          @click="togglePlaybackMode"
          class="mode-switch-btn btn-secondary animate-button-pop"
          :class="{ 'btn-primary': !isVideoMode, 'active': !isVideoMode }"
          :title="'音频模式'"
        >
          <svg class="mode-icon" viewBox="0 0 24 24">
            <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
          </svg>
          <span class="mode-text">音频</span>
        </button>
        
        <button 
          v-if="showVideoButton"
          @click="togglePlaybackMode"
          class="mode-switch-btn btn-secondary animate-button-pop"
          :class="{ 'btn-primary': isVideoMode, 'active': isVideoMode }"
          :title="'MV模式'"
          style="animation-delay: 0.1s;"
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
  padding: 1.25rem;
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.album-cover {
  width: 100%;
  max-width: 240px;
  margin-bottom: 1.25rem;
  position: relative;
  flex-shrink: 0;
}

.cover-container {
  width: 100%;
  padding-top: 100%; /* 1:1 Aspect Ratio */
  position: relative;
  border-radius: 50%;
  overflow: hidden;
  box-shadow: var(--shadow-lg);
  transform-origin: center center;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1), rgba(118, 75, 162, 0.1));
}

.cover-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: all var(--transition-normal);
}

.cover-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  backdrop-filter: blur(2px);
}

.play-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
}

.sound-waves {
  display: flex;
  gap: 3px;
  height: 24px;
  align-items: end;
}

.wave {
  width: 4px;
  background: var(--primary-gradient);
  border-radius: 2px;
  animation: wave 1.5s ease-in-out infinite;
  box-shadow: 0 0 8px rgba(102, 126, 234, 0.5);
}

.wave:nth-child(1) { animation-delay: 0s; }
.wave:nth-child(2) { animation-delay: 0.2s; }
.wave:nth-child(3) { animation-delay: 0.4s; }
.wave:nth-child(4) { animation-delay: 0.6s; }

@keyframes wave {
  0%, 100% { height: 8px; }
  50% { height: 24px; }
}

.song-details {
  text-align: center;
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: 0.625rem;
  min-height: 0;
}

.song-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin-bottom: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
  background: var(--primary-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.song-artist {
  font-size: 1rem;
  color: var(--text-secondary);
  margin-bottom: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.song-album {
  font-size: 0.9rem;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 0;
  /* 移除背景框和装饰效果 */
  /* background: var(--background-glass); */
  /* padding: 0.5rem 0.875rem; */
  /* border-radius: var(--radius-md); */
  /* backdrop-filter: blur(10px); */
}

.mode-switch-controls {
  display: flex;
  justify-content: center;
  gap: 0.625rem;
  margin-top: auto;
  padding-top: 1rem;
  width: 100%;
  flex-shrink: 0;
}

.mode-switch-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 0.875rem 1rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-normal);
  min-width: 80px;
  position: relative;
  overflow: hidden;
}

.mode-switch-btn.active {
  background: var(--primary-gradient);
  color: white;
  box-shadow: var(--shadow-primary);
  border-color: transparent;
}

.mode-switch-btn.active .mode-icon {
  fill: white;
}

.mode-icon {
  width: 22px;
  height: 22px;
  fill: var(--text-secondary);
  transition: all var(--transition-normal);
}

.mode-switch-btn:hover .mode-icon {
  transform: scale(1.1);
  fill: var(--primary-color);
}

.mode-switch-btn.active:hover .mode-icon {
  fill: white;
}

.mode-text {
  font-size: 0.85rem;
  font-weight: 600;
  white-space: nowrap;
}

/* NowPlaying组件内部动画 */
@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.8);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes slideUpStagger {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes buttonPop {
  from {
    opacity: 0;
    transform: scale(0.8) translateY(10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* 快速显示动画 - 移除延迟 */
.animate-scale-in {
  animation: scaleIn 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
}

.animate-slide-up-stagger {
  opacity: 0;
  animation: slideUpStagger 0.5s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.animate-slide-up-stagger .song-title {
  opacity: 0;
  animation: slideUpStagger 0.4s cubic-bezier(0.4, 0, 0.2, 1) 0.1s forwards;
}

.animate-slide-up-stagger .song-artist {
  opacity: 0;
  animation: slideUpStagger 0.4s cubic-bezier(0.4, 0, 0.2, 1) 0.2s forwards;
}

.animate-slide-up-stagger .song-album {
  opacity: 0;
  animation: slideUpStagger 0.4s cubic-bezier(0.4, 0, 0.2, 1) 0.3s forwards;
}

.animate-button-pop {
  opacity: 0;
  animation: buttonPop 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.4s forwards;
}

/* 响应式设计 */
@media (max-width: 480px) {
  .now-playing {
    padding: 1rem 0.875rem;
  }
  
  .album-cover {
    max-width: 160px;
    margin-bottom: 0.875rem;
  }
  
  .song-title {
    font-size: 1.1rem;
  }
  
  .song-artist {
    font-size: 0.9rem;
  }
  
  .song-album {
    font-size: 0.8rem;
    padding: 0.25rem 0.5rem;
  }
  
  .mode-switch-controls {
    gap: 0.375rem;
    padding-top: 0.625rem;
  }
  
  .mode-switch-btn {
    padding: 0.625rem 0.75rem;
    min-width: 65px;
  }
  
  .mode-icon {
    width: 18px;
    height: 18px;
  }
  
  .mode-text {
    font-size: 0.75rem;
  }
}

/* 触摸设备优化 */
@media (hover: none) and (pointer: coarse) {
  .mode-switch-btn {
    padding: 1rem 1.25rem;
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
