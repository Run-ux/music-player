import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface SongInfo {
  path: string;
  title?: string;
  artist?: string;
  album?: string;
  albumCover?: string;
  duration?: number;
}

export enum PlayerState {
  Playing = 'PLAYING',
  Paused = 'PAUSED',
  Stopped = 'STOPPED'
}

export enum PlayMode {
  Sequential = 'SEQUENTIAL',
  Repeat = 'REPEAT',
  Shuffle = 'SHUFFLE'
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
    await invoke('play');
    state.value = PlayerState.Playing;
  };
  
  const pause = async () => {
    await invoke('pause');
    state.value = PlayerState.Paused;
  };
  
  const stop = async () => {
    await invoke('stop');
    state.value = PlayerState.Stopped;
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
    await invoke('open_audio_file');
  };
  
  const updateProgress = (pos: number, dur: number) => {
    position.value = pos;
    duration.value = dur;
  };
  
  const updatePlaylist = (newPlaylist: SongInfo[]) => {
    playlist.value = newPlaylist;
  };
  
  const updateCurrentSong = (index: number) => {
    currentIndex.value = index;
  };
  
  const updateState = (newState: PlayerState) => {
    state.value = newState;
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
    stop,
    next,
    previous,
    setCurrentSong,
    addSong,
    removeSong,
    clearPlaylist,
    setPlayMode,
    openAudioFile,
    updateProgress,
    updatePlaylist,
    updateCurrentSong,
    updateState
  };
});
