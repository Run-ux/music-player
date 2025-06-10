<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed } from 'vue';

import { SongInfo } from '../stores/player';

const props = defineProps<{
  playlist: SongInfo[];
  currentIndex: number | null;
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
      <h3>播放列表</h3>      <div class="playlist-actions">
        <button @click="handleAddSongs" class="action-btn add">
          <span>添加歌曲</span>
        </button>
        <button @click="handleClearPlaylist" class="action-btn clear" :disabled="songCount === 0">
          <span>清空列表</span>
        </button>
      </div>
    </div>
      <div class="playlist-content">
      <div v-if="songCount === 0" class="empty-playlist">
        <p>播放列表为空，请添加歌曲</p>
      </div>
      <ul v-else class="song-list">
        <li 
          v-for="(song, index) in playlist" 
          :key="index"
          :class="{ 'current-song': index === currentIndex }"
          @click="handleSelectSong(index)"
        >
          <div class="song-info">
            <div class="song-title">{{ song.title || '未知歌曲' }}</div>
            <div class="song-artist">{{ song.artist || '未知艺术家' }}</div>
          </div>
          <div class="song-meta">
            <span class="song-duration">{{ formatDuration(song.duration) }}</span>
            <button @click="(e) => handleRemoveSong(e, index)" class="remove-btn">
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
  border-radius: 8px;
  background: #f9f9f9;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
}

.playlist-header {
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #eee;
}

.playlist-header h3 {
  margin: 0;
  font-weight: 500;
  color: #333;
}

.playlist-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.action-btn.add {
  background-color: #4caf50;
  color: white;
}

.action-btn.clear {
  background-color: #f44336;
  color: white;
}

.action-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.playlist-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.empty-playlist {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #888;
}

.song-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.song-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.song-list li:hover {
  background-color: #f0f0f0;
}

.song-list li.current-song {
  background-color: #e8f5e9;
  border-left: 3px solid #4caf50;
}

.song-info {
  flex: 1;
  overflow: hidden;
}

.song-title {
  font-weight: 500;
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-artist {
  font-size: 0.85rem;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.song-duration {
  font-size: 0.85rem;
  color: #555;
}

.remove-btn {
  background: none;
  border: none;
  color: #999;
  cursor: pointer;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  transition: all 0.2s;
}

.remove-btn:hover {
  background-color: rgba(0, 0, 0, 0.1);
  color: #f44336;
}
</style>
