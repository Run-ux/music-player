import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface LyricLine {
  time: number;    // 时间戳（毫秒）
  text: string;    // 歌词文本
}

export interface SongInfo {
  path: string;
  title?: string;
  artist?: string;
  album?: string;
  albumCover?: string;
  duration?: number;
  lyrics?: LyricLine[];  // 歌词信息
}

export enum PlayerState {
  Playing = 'Playing',
  Paused = 'Paused',
  Stopped = 'Stopped'
}

export enum PlayMode {
  Sequential = 'Sequential',
  Repeat = 'Repeat',
  Shuffle = 'Shuffle'
}

export const usePlayerStore = defineStore('player', () => {
  // 状态
  const state = ref<PlayerState>(PlayerState.Stopped);
  const playlist = ref<SongInfo[]>([]);
  const currentIndex = ref<number | null>(null);
  const playMode = ref<PlayMode>(PlayMode.Sequential);
  const position = ref<number>(0);
  const duration = ref<number>(0);
  
  // 计算属性
  const isPlaying = computed(() => state.value === PlayerState.Playing);
  const progress = computed(() => {
    if (!duration.value) return 0;
    return (position.value / duration.value) * 100;
  });
  
  const currentSong = computed(() => {
    if (currentIndex.value !== null && playlist.value.length > 0) {
      return playlist.value[currentIndex.value];
    }
    return null;
  });
  
  // 方法
  const play = async () => {
    // 如果没有选中歌曲且播放列表不为空，自动选择第一首歌曲
    if (currentIndex.value === null && playlist.value.length > 0) {
      await setCurrentSong(0);
    }
    
    // 如果还是没有歌曲可播放，直接返回
    if (currentIndex.value === null || playlist.value.length === 0) {
      console.warn('没有可播放的歌曲');
      return;
    }
    
    await invoke('play');
    state.value = PlayerState.Playing;
  };
  
  const pause = async () => {
    await invoke('pause');
    state.value = PlayerState.Paused;
  };
  
  const next = async () => {
    await invoke('next');
  };
  
  const previous = async () => {
    await invoke('previous');
  };
  
  const setCurrentSong = async (index: number) => {
    if (index >= 0 && index < playlist.value.length) {
      await invoke('set_song', { index });
      currentIndex.value = index;
    }
  };
  
  const addSong = async (path: string) => {
    await invoke('add_song', { path });
  };
  
  const removeSong = async (index: number) => {
    await invoke('remove_song', { index });
  };
  
  const clearPlaylist = async () => {
    await invoke('clear_playlist');
    playlist.value = [];
    currentIndex.value = null;
  };
  
  const setPlayMode = async (mode: PlayMode) => {
    await invoke('set_play_mode', { mode });
    playMode.value = mode;
  };  

  const openAudioFile = async () => {
    await invoke('open_audio_files');
  };

  // 添加跳转功能
  const seekTo = async (position: number) => {
    try {
      await invoke('seek_to', { position });
      console.log('跳转到位置:', position, '秒');
    } catch (error) {
      console.error('跳转失败:', error);
    }
  };
  
  const updateProgress = (pos: number, dur: number) => {
    position.value = pos;
    duration.value = dur;
  };

  // 添加专门的进度重置方法
  const resetProgress = () => {
    position.value = 0;
    duration.value = 0;
  };

  // 更新当前歌曲时自动重置进度
  const updateCurrentSong = (index: number) => {
    const oldIndex = currentIndex.value;
    currentIndex.value = index;
    
    // 如果歌曲索引发生变化，重置进度条
    if (oldIndex !== index) {
      resetProgress();
      console.log('歌曲索引变化，进度条重置:', index);
    }
  };

  const updatePlaylist = (newPlaylist: SongInfo[]) => {
    // 清空现有播放列表并重新赋值以确保响应性
    playlist.value.splice(0, playlist.value.length, ...newPlaylist);
  };
  
  const updateState = (newState: PlayerState) => {
    state.value = newState;
  };

  const updatePlayMode = (mode: PlayMode) => {
    playMode.value = mode;
  };
  
  return {
    // 状态
    state,
    playlist,
    currentIndex,
    playMode,
    position,
    duration,
    
    // 计算属性
    isPlaying,
    progress,
    currentSong,
    
    // 方法
    play,
    pause,
    next,
    previous,
    setCurrentSong,
    addSong,
    removeSong,
    clearPlaylist,
    setPlayMode,
    openAudioFile,
    seekTo,
    updateProgress,
    updatePlaylist,
    updateCurrentSong,
    updateState,
    updatePlayMode
  };
});
