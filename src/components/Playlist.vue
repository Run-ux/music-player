<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed } from 'vue';

import { SongInfo } from '../stores/player';

const props = defineProps<{
  playlist: SongInfo[];
  currentIndex: number | null;
  isPlaying: boolean;
}>();

// 使用计算属性，确保TypeScript知道props被使用
const songCount = computed(() => props.playlist.length);

const emit = defineEmits<{
  selectSong: [index: number];
  removeSong: [index: number];
}>();

// 选择歌曲播放
const handleSelectSong = (index: number) => {
  emit('selectSong', index);
};

// 从播放列表中移除歌曲
const handleRemoveSong = (event: Event, index: number) => {
  event.stopPropagation();
  emit('removeSong', index);
};

// 打开添加歌曲对话框
const handleAddSongs = async () => {
  await invoke('open_audio_files');
};

// 清空播放列表 - 修改为确认后再清空
const handleClearPlaylist = async () => {
  // 使用原生确认对话框
  const confirmed = confirm('确定要清空播放列表吗？');
  
  if (confirmed) {
    try {
      await invoke('clear_playlist');
      console.log('播放列表已清空');
    } catch (error) {
      console.error('清空播放列表失败:', error);
    }
  } else {
    console.log('用户取消清空播放列表');
  }
};

// 计算歌曲时长显示
const formatDuration = (seconds: number | undefined) => {
  if (!seconds) return '--:--';
  
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
};
</script>

<template>
  <div class="playlist card">
    <div class="playlist-header">
      <div class="header-content">
        <div class="header-title">
          <h3>🎵 播放列表</h3>
          <span class="song-count">{{ songCount }} 首歌曲</span>
        </div>
        <div class="playlist-actions">
          <button @click="handleAddSongs" class="btn btn-success">
            <span class="btn-icon">➕</span>
            <span>添加歌曲</span>
          </button>
          <button @click="handleClearPlaylist" class="btn btn-danger" :disabled="songCount === 0">
            <span class="btn-icon">🗑️</span>
            <span>清空列表</span>
          </button>
        </div>
      </div>
    </div>
    
    <div class="playlist-content">
      <div v-if="songCount === 0" class="empty-playlist">
        <div class="empty-icon">🎵</div>
        <h4>播放列表为空</h4>
        <p>点击"添加歌曲"开始您的音乐之旅</p>
        <button @click="handleAddSongs" class="btn btn-primary">
          <span class="btn-icon">➕</span>
          添加歌曲
        </button>
      </div>
      
      <ul v-else class="song-list">
        <li 
          v-for="(song, index) in playlist" 
          :key="index"
          :class="{ 'current-song': index === currentIndex, 'playing': index === currentIndex && isPlaying }"
          @click="handleSelectSong(index)"
          class="song-item"
        >
          <div class="song-status">
            <div 
              v-if="index === currentIndex" 
              class="play-indicator"
              :class="{ playing: isPlaying }"
            >
              <div class="play-icon">
                {{ isPlaying ? '▶️' : '⏸️' }}
              </div>
              <div class="sound-waves" v-if="isPlaying">
                <div class="wave"></div>
                <div class="wave"></div>
                <div class="wave"></div>
              </div>
            </div>
            <span v-else class="song-number">{{ String(index + 1).padStart(2, '0') }}</span>
          </div>
          
          <div class="song-cover">
            <div class="cover-placeholder">
              <span class="media-type-icon" v-if="song.mediaType === 'Video'">🎬</span>
              <span class="media-type-icon" v-else>🎵</span>
            </div>
          </div>
          
          <div class="song-info">
            <div class="song-title">
              {{ song.title || '未知歌曲' }}
            </div>
            <div class="song-artist">
              {{ song.artist || (song.mediaType === 'Video' ? '视频文件' : '未知艺术家') }}
            </div>
          </div>
          
          <div class="song-meta">
            <span class="song-duration">{{ formatDuration(song.duration) }}</span>
            <button @click="(e) => handleRemoveSong(e, index)" class="remove-btn" title="移除歌曲">
              <span>✕</span>
            </button>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.playlist {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.playlist-header {
  padding: 1rem;
  background: var(--background-glass);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border-light);
  position: relative;
  flex-shrink: 0;
}

.playlist-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.08), rgba(118, 75, 162, 0.08));
  z-index: -1;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.header-title h3 {
  margin: 0;
  font-weight: 700;
  font-size: 1.25rem;
  background: var(--primary-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.song-count {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-top: 0.25rem;
  display: block;
  font-weight: 500;
}

.playlist-actions {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.playlist-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  min-height: 0;
}

.playlist-content::-webkit-scrollbar {
  width: 6px;
}

.playlist-content::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 3px;
}

.playlist-content::-webkit-scrollbar-thumb {
  background: var(--primary-gradient);
  border-radius: 3px;
}

.empty-playlist {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 300px;
  text-align: center;
  color: var(--text-secondary);
  padding: 2rem;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.6;
  background: var(--primary-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.empty-playlist h4 {
  margin: 0 0 0.75rem 0;
  font-size: 1.25rem;
  color: var(--text-primary);
  font-weight: 600;
}

.empty-playlist p {
  margin: 0 0 1.5rem 0;
  color: var(--text-secondary);
  font-size: 1rem;
}

.song-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.song-item {
  display: flex;
  align-items: center;
  padding: 0.75rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-normal);
  gap: 0.75rem;
  background: var(--background-secondary);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border-light);
  position: relative;
  overflow: hidden;
}

.song-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05), rgba(118, 75, 162, 0.05));
  opacity: 0;
  transition: opacity var(--transition-normal);
}

.song-item:hover::before {
  opacity: 1;
}

.song-item:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--border-primary);
}

.song-item.current-song {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1), rgba(118, 75, 162, 0.1));
  border-color: var(--border-primary);
  box-shadow: var(--shadow-primary);
}

.song-item.playing {
  animation: playingGlow 2s ease-in-out infinite alternate;
}

@keyframes playingGlow {
  from {
    box-shadow: var(--shadow-primary);
  }
  to {
    box-shadow: 0 6px 25px rgba(102, 126, 234, 0.4);
  }
}

.song-status {
  width: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  position: relative;
}

.song-number {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-muted);
}

.play-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.play-icon {
  font-size: 1rem;
  color: var(--primary-color);
}

.sound-waves {
  display: flex;
  gap: 2px;
  height: 12px;
  align-items: end;
}

.wave {
  width: 3px;
  background: var(--primary-gradient);
  border-radius: 2px;
  animation: wave 1.5s ease-in-out infinite;
}

.wave:nth-child(1) { animation-delay: 0s; }
.wave:nth-child(2) { animation-delay: 0.2s; }
.wave:nth-child(3) { animation-delay: 0.4s; }

@keyframes wave {
  0%, 100% { height: 4px; }
  50% { height: 12px; }
}

.song-cover {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, var(--background-glass), rgba(102, 126, 234, 0.1));
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-sm);
  border: 1px solid var(--border-light);
}

.media-type-icon {
  font-size: 1.25rem;
  opacity: 0.7;
}

.song-info {
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.song-title {
  font-weight: 600;
  margin-bottom: 0.125rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
  font-size: 0.9rem;
  line-height: 1.2;
}

.song-artist {
  font-size: 0.8rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

.song-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
}

.song-duration {
  font-size: 0.8rem;
  color: var(--text-secondary);
  font-weight: 500;
  background: var(--background-glass);
  padding: 0.25rem 0.375rem;
  border-radius: var(--radius-sm);
  backdrop-filter: blur(5px);
  line-height: 1;
}

.remove-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
}

.remove-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 50%;
  transform: scale(0);
  transition: transform var(--transition-normal);
}

.remove-btn:hover::before {
  transform: scale(1);
}

.remove-btn:hover {
  color: var(--danger-color);
  transform: scale(1.1);
}

.remove-btn span {
  position: relative;
  z-index: 1;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .playlist-header {
    padding: 0.75rem;
  }
  
  .header-content {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
  }
  
  .playlist-actions {
    justify-content: center;
  }
  
  .playlist-content {
    padding: 0.5rem;
  }
  
  .song-item {
    padding: 0.625rem;
    gap: 0.625rem;
  }
  
  .song-cover {
    width: 36px;
    height: 36px;
  }
  
  .media-type-icon {
    font-size: 1.1rem;
  }
  
  .song-title {
    font-size: 0.85rem;
  }
  
  .song-artist {
    font-size: 0.75rem;
  }
  
  .song-duration {
    font-size: 0.75rem;
    padding: 0.2rem 0.3rem;
  }
  
  .remove-btn {
    width: 26px;
    height: 26px;
    font-size: 0.85rem;
  }
}
</style>
