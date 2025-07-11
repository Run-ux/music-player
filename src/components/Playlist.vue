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

// 清空播放列表
const handleClearPlaylist = async () => {
  if (confirm('确定要清空播放列表吗？')) {
    await invoke('clear_playlist');
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
  <div class="playlist">
    <div class="playlist-header">
      <div class="header-content">
        <div class="header-title">
          <h3>🎵 播放列表</h3>
          <span class="song-count">{{ songCount }} 首歌曲</span>
        </div>
        <div class="playlist-actions">
          <button @click="handleAddSongs" class="action-btn add">
            <span class="btn-icon">➕</span>
            <span>添加歌曲</span>
          </button>
          <button @click="handleClearPlaylist" class="action-btn clear" :disabled="songCount === 0">
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
        <button @click="handleAddSongs" class="empty-add-btn">
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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 16px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  position: relative;
}

.playlist::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  z-index: 1;
}

.playlist > * {
  position: relative;
  z-index: 2;
}

.playlist-header {
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.header-title h3 {
  margin: 0;
  font-weight: 600;
  font-size: 1.25rem;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.song-count {
  font-size: 0.85rem;
  color: #666;
  margin-top: 0.25rem;
  display: block;
}

.playlist-actions {
  display: flex;
  gap: 0.75rem;
}

.action-btn {
  padding: 0.75rem 1rem;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
  overflow: hidden;
}

.action-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  transition: left 0.5s;
}

.action-btn:hover::before {
  left: 100%;
}

.action-btn.add {
  background: linear-gradient(135deg, #4ade80, #22c55e);
  color: white;
  box-shadow: 0 4px 15px rgba(34, 197, 94, 0.3);
}

.action-btn.clear {
  background: linear-gradient(135deg, #f87171, #ef4444);
  color: white;
  box-shadow: 0 4px 15px rgba(239, 68, 68, 0.3);
}

.action-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
}

.action-btn:disabled {
  background: #e5e7eb;
  color: #9ca3af;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-icon {
  font-size: 1rem;
}

.playlist-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.playlist-content::-webkit-scrollbar {
  width: 6px;
}

.playlist-content::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 3px;
}

.playlist-content::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 3px;
}

.empty-playlist {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  color: #6b7280;
  padding: 2rem;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-playlist h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: #374151;
}

.empty-playlist p {
  margin: 0 0 1.5rem 0;
  color: #6b7280;
}

.empty-add-btn {
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.3s;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.empty-add-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
}

.song-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.song-list li {
  display: flex;
  align-items: center;
  padding: 1rem;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  gap: 1rem;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  position: relative;
  overflow: hidden;
}

.song-list li::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1), rgba(118, 75, 162, 0.1));
  opacity: 0;
  transition: opacity 0.3s;
}

.song-list li:hover::before {
  opacity: 1;
}

.song-list li:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.song-list li.current-song {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.15), rgba(118, 75, 162, 0.15));
  border: 1px solid rgba(102, 126, 234, 0.3);
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.2);
}

.song-list li.playing {
  animation: playingGlow 2s ease-in-out infinite alternate;
}

@keyframes playingGlow {
  from {
    box-shadow: 0 4px 20px rgba(102, 126, 234, 0.2);
  }
  to {
    box-shadow: 0 4px 30px rgba(102, 126, 234, 0.4);
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
  color: #6b7280;
}

.play-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.play-icon {
  font-size: 1rem;
  color: #667eea;
}

.sound-waves {
  display: flex;
  gap: 2px;
  height: 12px;
  align-items: end;
}

.wave {
  width: 3px;
  background: linear-gradient(to top, #667eea, #764ba2);
  border-radius: 2px;
  animation: wave 1.5s ease-in-out infinite;
}

.wave:nth-child(1) {
  animation-delay: 0s;
}

.wave:nth-child(2) {
  animation-delay: 0.2s;
}

.wave:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes wave {
  0%, 100% {
    height: 4px;
  }
  50% {
    height: 12px;
  }
}

.song-cover {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.media-type-icon {
  font-size: 1.5rem;
  opacity: 0.6;
}

.song-info {
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.song-title {
  font-weight: 600;
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #1f2937;
  font-size: 0.95rem;
}

.song-artist {
  font-size: 0.85rem;
  color: #6b7280;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-meta {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-shrink: 0;
}

.song-duration {
  font-size: 0.85rem;
  color: #6b7280;
  font-weight: 500;
  background: rgba(107, 114, 128, 0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 6px;
}

.remove-btn {
  background: none;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  font-size: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  transition: all 0.3s;
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
  transition: transform 0.3s;
}

.remove-btn:hover::before {
  transform: scale(1);
}

.remove-btn:hover {
  color: #ef4444;
  transform: scale(1.1);
}

.remove-btn span {
  position: relative;
  z-index: 1;
}
</style>
