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
  duration?: number; // 秒
  lyrics?: LyricLine[];
  mediaType?: MediaType;
  mvPath?: string;
  videoThumbnail?: string;
  hasLyrics?: boolean;
  // 新增：支持播放模式切换判断
  supportsModeSwitch?: boolean;
  isPureVideo?: boolean;
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
  const currentPlaybackMode = ref<MediaType>(MediaType.Audio); // 新增：当前播放模式
  
  // 新增：智能播放状态检测
  const isActuallyPlaying = ref(false); // 真实播放状态
  const lastPositionUpdate = ref(0); // 最后一次进度更新时间
  const isTransitioning = ref(false); // 是否正在跳转
  const lastPosition = ref(0); // 上次记录的播放位置
  const positionUpdateCount = ref(0); // 进度更新计数器
  const isNewSong = ref(false); // 是否是新歌曲开始
  
  // 关键修复：简化播放状态检测逻辑
  const isReallyPlaying = computed(() => {
    // 基础判断：如果状态不是播放，肯定不在播放
    if (state.value !== PlayerState.Playing) return false;
    
    // 如果正在跳转，使用设定的播放状态
    if (isTransitioning.value) return true;
    
    // 关键修复：立即信任后端状态，减少复杂的前端检测
    return true;
  });
  
  // 新增：音视频互斥播放控制
  const isVideoPlayerActive = ref(false); // 视频播放器是否激活
  const isAudioPlayerActive = ref(false); // 音频播放器是否激活
  const mutexLock = ref(false); // 互斥锁，防止同时播放
  
  // 新增：全局播放状态检查
  const checkPlaybackState = () => {
    const hasActivePlayer = isVideoPlayerActive.value || isAudioPlayerActive.value;
    const shouldBePlaying = state.value === PlayerState.Playing;
    
    if (shouldBePlaying && !hasActivePlayer) {
      console.warn('⚠️ 播放状态不一致：应该播放但没有激活的播放器');
      // 根据当前歌曲类型重新激活对应播放器
      const current = currentSong.value;
      if (current) {
        const isVideoMode = current.mediaType === MediaType.Video || 
                           (currentPlaybackMode.value === MediaType.Video && current.mvPath);
        if (isVideoMode) {
          activateVideoPlayer();
        } else {
          activateAudioPlayer();
        }
      }
    } else if (!shouldBePlaying && hasActivePlayer) {
      console.warn('⚠️ 播放状态不一致：不应该播放但有激活的播放器');
      deactivateVideoPlayer();
      deactivateAudioPlayer();
    }
  };
  
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
  
  // 增强：音视频互斥控制方法
  const activateVideoPlayer = () => {
    if (mutexLock.value) {
      console.log('🔒 互斥锁激活中，等待解锁...');
      return false;
    }
    
    mutexLock.value = true;
    
    // 强制停止音频播放器
    if (isAudioPlayerActive.value) {
      console.log('🎵➡️🎬 强制停止音频播放，激活视频播放器');
      isAudioPlayerActive.value = false;
      // 立即调用后端停止音频
      invoke('pause').catch(console.error);
    }
    
    isVideoPlayerActive.value = true;
    console.log('✅ 视频播放器已激活');
    
    // 延迟解锁，确保状态切换完成
    setTimeout(() => {
      mutexLock.value = false;
      // 解锁后进行状态检查
      setTimeout(checkPlaybackState, 100);
    }, 200);
    
    return true;
  };
  
  // 关键修复：优化音频播放器激活逻辑，减少不必要的暂停
  const activateAudioPlayer = () => {
    if (mutexLock.value) {
      console.log('🔒 互斥锁激活中，等待解锁...');
      return false;
    }
    
    mutexLock.value = true;
    
    // 关键修复：只有在视频播放器真正在播放时才强制停止
    if (isVideoPlayerActive.value) {
      console.log('🎬➡️🎵 检测到视频播放器激活，停止视频播放');
      isVideoPlayerActive.value = false;
      // 发送信号给VideoPlayer组件停止播放
      // 这通过响应式状态变化来实现
    }
    
    isAudioPlayerActive.value = true;
    console.log('✅ 音频播放器已激活');
    
    // 缩短解锁时间，提高响应速度
    setTimeout(() => {
      mutexLock.value = false;
      // 解锁后进行状态检查
      setTimeout(checkPlaybackState, 50);
    }, 100);
    
    return true;
  };
  
  const deactivateVideoPlayer = () => {
    if (isVideoPlayerActive.value) {
      isVideoPlayerActive.value = false;
      console.log('🎬 视频播放器已停用');
    }
  };
  
  const deactivateAudioPlayer = () => {
    if (isAudioPlayerActive.value) {
      isAudioPlayerActive.value = false;
      console.log('🎵 音频播放器已停用');
    }
  };
  
  // 新增：强制停止所有播放器
  const stopAllPlayers = async () => {
    console.log('🛑 强制停止所有播放器');
    
    mutexLock.value = true;
    
    // 停止视频播放器
    if (isVideoPlayerActive.value) {
      deactivateVideoPlayer();
    }
    
    // 停止音频播放器
    if (isAudioPlayerActive.value) {
      deactivateAudioPlayer();
      try {
        await invoke('pause');
      } catch (error) {
        console.error('停止音频播放失败:', error);
      }
    }
    
    setTimeout(() => {
      mutexLock.value = false;
    }, 100);
  };
  
  // 方法
  const play = async () => {
    console.log('🎮 开始播放流程');
    
    // 如果没有选中歌曲且播放列表不为空，自动选择第一首歌曲
    if (currentIndex.value === null && playlist.value.length > 0) {
      await setCurrentSong(0);
    }
    
    // 如果还是没有歌曲可播放，直接返回
    if (currentIndex.value === null || playlist.value.length === 0) {
      console.warn('没有可播放的歌曲');
      return;
    }
    
    try {
      // 检查当前播放模式，决定激活哪个播放器
      const current = currentSong.value;
      const isVideoMode = current?.mediaType === MediaType.Video || 
                         (currentPlaybackMode.value === MediaType.Video && current?.mvPath);
      
      console.log('🎯 播放模式判断:', {
        isVideoMode,
        mediaType: current?.mediaType,
        playbackMode: currentPlaybackMode.value,
        hasMv: !!current?.mvPath
      });
      
      if (isVideoMode) {
        // 视频模式：激活视频播放器
        console.log('🎬 激活视频播放器');
        if (!activateVideoPlayer()) {
          console.error('视频播放器激活失败');
          return;
        }
      } else {
        // 音频模式：激活音频播放器并调用后端
        console.log('🎵 激活音频播放器');
        if (!activateAudioPlayer()) {
          console.error('音频播放器激活失败');
          return;
        }
        
        // 调用后端播放
        await invoke('play');
        console.log('✅ 后端音频播放命令发送成功');
      }
      
      // 立即设置播放状态
      state.value = PlayerState.Playing;
      console.log('✅ 播放流程完成');
    } catch (error) {
      console.error('播放失败:', error);
      // 失败时清理状态
      deactivateVideoPlayer();
      deactivateAudioPlayer();
      state.value = PlayerState.Paused;
    }
  };
  
  const pause = async () => {
    console.log('⏸️ 开始暂停流程');
    
    try {
      // 简化暂停逻辑：直接调用后端暂停，不管当前是什么模式
      await invoke('pause');
      
      // 立即设置暂停状态
      state.value = PlayerState.Paused;
      
      // 停用所有播放器
      deactivateVideoPlayer();
      deactivateAudioPlayer();
      
      console.log('✅ 暂停流程完成');
    } catch (error) {
      console.error('暂停失败:', error);
    }
  };
  
  const next = async () => {
    console.log('⏭️ 切换到下一首歌曲');
    
    // 切歌前先停止所有播放器
    await stopAllPlayers();
    
    try {
      await invoke('next');
      // 重要：确保前端状态也更新为播放状态，因为后端在切换时会自动开始播放
      state.value = PlayerState.Playing;
      console.log('✅ 下一首切换完成');
    } catch (error) {
      console.error('切换下一首失败:', error);
    }
  };
  
  const previous = async () => {
    console.log('⏮️ 切换到上一首歌曲');
    
    // 切歌前先停止所有播放器
    await stopAllPlayers();
    
    try {
      await invoke('previous');
      // 重要：确保前端状态也更新为播放状态，因为后端在切换时会自动开始播放
      state.value = PlayerState.Playing;
      console.log('✅ 上一首切换完成');
    } catch (error) {
      console.error('切换上一首失败:', error);
    }
  };
  
  const setCurrentSong = async (index: number) => {
    if (index >= 0 && index < playlist.value.length) {
      console.log('🎵 用户选择歌曲:', index, playlist.value[index]?.title);
      
      // 选歌前先停止所有播放器
      await stopAllPlayers();
      
      try {
        await invoke('set_song', { index });
        currentIndex.value = index;
        // 重要：确保前端状态也更新为播放状态，因为后端在设置歌曲时会自动开始播放
        state.value = PlayerState.Playing;
        console.log('✅ 歌曲选择完成');
      } catch (error) {
        console.error('选择歌曲失败:', error);
      }
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

  // 完全重写seekTo方法，彻底分离音频和视频跳转逻辑
  const seekTo = async (targetPosition: number) => {
    try {
      const current = currentSong.value;
      if (!current) {
        console.warn('没有当前歌曲，无法跳转');
        return;
      }

      console.log('🎯 智能跳转开始:', targetPosition, '秒，当前歌曲:', current.title);
      
      // 设置跳转状态，防止其他事件干扰
      setTransitioning(true);
      
      // 检查是否是视频模式
      const isVideoMode = current.mediaType === MediaType.Video || 
                         (currentPlaybackMode.value === MediaType.Video && current.mvPath);
      
      if (isVideoMode) {
        console.log('🎬 视频模式跳转 - 完全由前端VideoPlayer处理');
        
        // 立即更新前端进度，给用户即时反馈
        position.value = targetPosition;
        
        // 关键修复：通知VideoPlayer组件进行跳转，但不调用后端API
        // VideoPlayer会监听position变化并执行视频跳转
        
        // 延迟重置状态，给VideoPlayer足够时间处理
        setTimeout(() => {
          setTransitioning(false);
          console.log('视频跳转流程完成');
        }, 800);
      } else {
        // 音频模式：正常调用后端跳转
        console.log('🎵 音频模式跳转 - 调用后端API');
        
        // 立即更新前端进度，给用户即时反馈
        position.value = targetPosition;
        
        // 只有音频模式才调用后端跳转
        await invoke('seek_to', { position: targetPosition });
        console.log('后端音频跳转完成');
        
        // 延迟重置状态
        setTimeout(() => {
          setTransitioning(false);
        }, 500);
      }
    } catch (error) {
      console.error('跳转失败:', error);
      setTransitioning(false);
    }
  };
  
  const updateProgress = (pos: number, dur: number) => {
    // 关键修复：简化进度更新逻辑，避免复杂的状态检测导致播放键跳跃
    position.value = pos;
    duration.value = dur;
    
    // 关键修复：移除复杂的播放状态检测逻辑
    // 直接信任后端状态，避免前端过度干预导致状态不一致
    
    // 只保留必要的新歌曲检测
    if (pos === 0 && dur !== duration.value) {
      isNewSong.value = true;
      console.log('检测到新歌曲开始');
    } else if (pos > 2) {
      isNewSong.value = false;
    }
    
    // 关键修复：移除自动状态修正逻辑，避免干扰用户操作
    // 让后端完全控制播放状态，前端只负责显示
    
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
  
  // 新增：切换播放模式的方法
  const togglePlaybackMode = async () => {
    await invoke('toggle_playback_mode');
    // 切换后更新本地状态
    const newMode = currentPlaybackMode.value === MediaType.Audio ? MediaType.Video : MediaType.Audio;
    currentPlaybackMode.value = newMode;
    console.log('播放模式已切换为:', newMode);
  };

  // 简化的播放模式切换方法
  const setPlaybackMode = async (mode: MediaType) => {
    console.log('前端设置播放模式:', mode);
    
    // 记录切换前的状态
    const wasPlaying = isPlaying.value;
    const oldMode = currentPlaybackMode.value;
    
    try {
      // 调用后端设置播放模式
      await invoke('set_playback_mode', { mode });
      
      // 立即更新本地状态
      currentPlaybackMode.value = mode;
      console.log('播放模式已设置为:', mode);
      
      // 关键修复：视频切音频时确保播放状态和UI同步
      if (oldMode === MediaType.Video && mode === MediaType.Audio) {
        console.log('🎵 视频切音频模式，强制确保播放状态同步');
        
        // 立即设置为播放状态，确保播放按钮显示正确
        state.value = PlayerState.Playing;
        
        // 等待一小段时间让后端处理完成
        await new Promise(resolve => setTimeout(resolve, 100));
        
        // 确保音频开始播放
        try {
          await invoke('play');
          console.log('✅ 视频切音频后音频自动开始播放');
          
          // 再次确认播放状态
          state.value = PlayerState.Playing;
        } catch (error) {
          console.warn('视频切音频后启动播放失败:', error);
        }
      } else if (oldMode === MediaType.Audio && mode === MediaType.Video) {
        console.log('🎬 音频切视频模式，确保状态同步');
        
        // 视频模式也确保播放状态
        if (wasPlaying) {
          state.value = PlayerState.Playing;
        }
      }
    } catch (error) {
      console.error('设置播放模式失败:', error);
      // 回滚本地状态
      currentPlaybackMode.value = oldMode;
    }
  };

  // 初始化时获取当前播放模式
  const initializePlaybackMode = async () => {
    try {
      const mode = await invoke('get_current_playback_mode') as MediaType;
      currentPlaybackMode.value = mode;
    } catch (error) {
      console.warn('获取播放模式失败，使用默认音频模式:', error);
      currentPlaybackMode.value = MediaType.Audio;
    }
  };

  // 检查当前歌曲是否有MV
  const currentSongHasMv = computed(() => {
    return currentSong.value?.mvPath !== undefined && currentSong.value?.mvPath !== null;
  });

  return {
    // 状态
    state,
    playlist,
    currentIndex,
    playMode,
    position,
    duration,
    currentPlaybackMode, // 新增
    
    // 新增状态
    isReallyPlaying, // 智能播放状态
    isTransitioning, // 跳转状态
    isNewSong, // 新歌曲状态
    isVideoPlayerActive, // 视频播放器激活状态
    isAudioPlayerActive, // 音频播放器激活状态
    
    // 计算属性
    isPlaying,
    progress,
    currentSong,
    currentSongHasMv, // 新增
    
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
    togglePlaybackMode, // 新增：切换播放模式
    setPlaybackMode,    // 新增：设置播放模式
    initializePlaybackMode, // 新增：初始化播放模式
    // 新增：音视频互斥控制方法
    activateVideoPlayer,
    activateAudioPlayer,
    deactivateVideoPlayer,
    deactivateAudioPlayer,
  };
});
