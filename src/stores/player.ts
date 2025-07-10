import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface LyricLine {
  time: number;    // 时间戳（毫秒）
  text: string;    // 歌词文本
}

// 新增：媒体类型枚举
export enum MediaType {
  Audio = 'Audio',
  Video = 'Video'
}

export interface SongInfo {
  path: string;
  title?: string;
  artist?: string;
  album?: string;
  albumCover?: string;
  duration?: number;
  lyrics?: LyricLine[];  // 歌词信息
  // 新增：MV相关字段
  mediaType?: MediaType;  // 媒体类型
  mvPath?: string;        // MV视频文件路径
  videoThumbnail?: string; // 视频缩略图
  hasLyrics?: boolean;    // 是否有歌词
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
  
  // 新增：智能播放状态检测
  const isActuallyPlaying = ref(false); // 真实播放状态
  const lastPositionUpdate = ref(0); // 最后一次进度更新时间
  const isTransitioning = ref(false); // 是否正在跳转
  const lastPosition = ref(0); // 上次记录的播放位置
  const positionUpdateCount = ref(0); // 进度更新计数器
  const isNewSong = ref(false); // 是否是新歌曲开始
  
  // 计算属性
  const isPlaying = computed(() => state.value === PlayerState.Playing);
  
  // 新增：智能检测是否真正在播放（更精确的逻辑）
  const isReallyPlaying = computed(() => {
    // 如果不是播放状态，肯定不在播放
    if (!isPlaying.value) return false;
    
    // 如果正在跳转，不显示播放状态
    if (isTransitioning.value) return false;
    
    // 如果是新歌曲且播放很快开始，直接显示播放状态
    if (isNewSong.value && isActuallyPlaying.value) {
      return true;
    }
    
    // 对于跳转后的情况，需要更严格的检测
    return isActuallyPlaying.value && positionUpdateCount.value >= 2;
  });
  
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
  const seekVideoTo = async (position: number) => {
    // 直接更新前端进度显示，给用户即时反馈
    position.value = position;
    console.log('视频跳转请求:', position, '秒');
  };

  // 修改原有的seekTo方法，增加视频文件检测
  const seekTo = async (position: number) => {
    try {
      // 检查当前是否为视频文件
      const current = currentSong.value;
      if (current?.mediaType === MediaType.Video) {
        // 视频文件：使用专门的视频跳转方法
        seekVideoTo(position);
        return;
      }
      
      // 音频文件：使用原有的后端跳转逻辑
      setTransitioning(true);
      await invoke('seek_to', { position });
      console.log('跳转到位置:', position, '秒');
      
      // 跳转后延迟恢复状态检测，给音频处理更多时间
      setTimeout(() => {
        setTransitioning(false);
      }, 800);
    } catch (error) {
      console.error('跳转失败:', error);
      setTransitioning(false);
    }
  };
  
  const updateProgress = (pos: number, dur: number) => {
    const now = Date.now();
    
    position.value = pos;
    duration.value = dur;
    
    // 检测是否是新歌曲（进度从0开始且持续时间发生变化）
    if (pos === 0 && dur !== duration.value) {
      isNewSong.value = true;
      positionUpdateCount.value = 0;
      console.log('检测到新歌曲开始');
    } else if (pos > 2) { // 播放超过2秒后不再认为是新歌
      isNewSong.value = false;
    }
    
    // 对于视频文件，使用更宽松的播放检测逻辑
    const currentSong = playlist.value[currentIndex.value || 0];
    const isVideo = currentSong?.mediaType === MediaType.Video;
    
    // 检测进度是否在更新（说明真正在播放）
    if (pos > lastPosition.value && pos > 0) {
      isActuallyPlaying.value = true;
      lastPositionUpdate.value = now;
      positionUpdateCount.value++;
      
      // 视频文件或新歌曲快速开始播放的情况
      if ((isVideo || isNewSong.value) && positionUpdateCount.value >= 1) {
        console.log(isVideo ? '视频播放状态确认' : '新歌曲快速开始播放');
      }
    } else if (Math.abs(pos - lastPosition.value) > 1) {
      // 如果位置跳跃很大（可能是跳转），重置计数器
      positionUpdateCount.value = 0;
      isActuallyPlaying.value = false;
      console.log('检测到位置跳跃，重置播放状态');
    } else if (now - lastPositionUpdate.value > (isVideo ? 3000 : 2000)) {
      // 视频文件给予更长的检测时间（3秒 vs 2秒）
      isActuallyPlaying.value = false;
      positionUpdateCount.value = 0;
    }
    
    lastPosition.value = pos;
  };

  // 优化：设置跳转状态
  const setTransitioning = (transitioning: boolean) => {
    isTransitioning.value = transitioning;
    if (transitioning) {
      // 跳转时重置播放检测
      isActuallyPlaying.value = false;
      positionUpdateCount.value = 0;
      console.log('开始跳转，重置播放状态检测');
    } else {
      console.log('跳转结束，开始检测播放状态');
    }
  };

  // 添加专门的进度重置方法
  const resetProgress = () => {
    position.value = 0;
    duration.value = 0;
    // 重置播放状态检测
    isActuallyPlaying.value = false;
    lastPositionUpdate.value = 0;
    lastPosition.value = 0;
    positionUpdateCount.value = 0;
    isNewSong.value = true; // 新歌曲标记
  };

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
  
  // 添加视频时长管理
  const videoDurations = ref<Map<string, number>>(new Map());

  // 更新视频文件的真实时长
  const updateVideoDuration = (filePath: string, duration: number) => {
    videoDurations.value.set(filePath, duration);
    console.log('更新视频时长缓存:', filePath, '→', duration, '秒');
    
    // 同时更新播放列表中对应歌曲的时长显示
    const songIndex = playlist.value.findIndex(song => song.path === filePath);
    if (songIndex !== -1) {
      // 创建新的歌曲对象，更新时长信息
      const updatedSong = { ...playlist.value[songIndex], duration };
      playlist.value[songIndex] = updatedSong;
      console.log('更新播放列表中的视频时长:', updatedSong.title, '→', duration, '秒');
    }
  };

  // 获取视频文件的真实时长
  const getVideoDuration = (filePath: string): number | undefined => {
    return videoDurations.value.get(filePath);
  };
  
  return {
    // 状态
    state,
    playlist,
    currentIndex,
    playMode,
    position,
    duration,
    
    // 新增状态
    isReallyPlaying, // 智能播放状态
    isTransitioning, // 跳转状态
    isNewSong, // 新歌曲状态
    
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
    updatePlayMode,
    setTransitioning, // 新增方法
    updateVideoDuration, // 更新视频时长
    getVideoDuration,     // 获取视频时长
  };
});
