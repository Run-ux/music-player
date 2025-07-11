<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { LyricLine } from '../stores/player';

const props = defineProps<{
  lyrics?: LyricLine[];
  currentTime: number; // 当前播放时间（秒）
  isPlaying: boolean;
}>();

const lyricsContainer = ref<HTMLElement>();
const currentLyricIndex = ref(-1);

// 计算当前应该高亮的歌词行
const currentLyric = computed(() => {
  if (!props.lyrics || props.lyrics.length === 0) {
    return null;
  }

  // 添加时间偏移量，让歌词提前显示（毫秒）
  const LYRIC_OFFSET_MS = 1100; // 提前1100毫秒显示歌词
  const currentTimeMs = props.currentTime * 1000 + LYRIC_OFFSET_MS; // 转换为毫秒并添加偏移
  let index = -1;

  console.log(`计算歌词高亮: 当前时间 ${props.currentTime}秒 + 偏移 ${LYRIC_OFFSET_MS}ms = ${currentTimeMs}ms`);

  for (let i = 0; i < props.lyrics.length; i++) {
    const currentLyricTime = props.lyrics[i].time;
    const nextLyricTime = i < props.lyrics.length - 1 ? props.lyrics[i + 1].time : Infinity;
    
    console.log(`检查歌词 ${i}: ${currentLyricTime}ms - ${nextLyricTime}ms, 内容: "${props.lyrics[i].text}"`);
    
    // 如果当前时间在这一行歌词的时间范围内，就选择这一行
    if (currentTimeMs >= currentLyricTime && currentTimeMs < nextLyricTime) {
      index = i;
      console.log(`✅ 找到匹配歌词: 索引 ${i}, 内容: "${props.lyrics[i].text}"`);
      break;
    }
  }

  // 如果没有找到匹配的歌词行
  if (index === -1) {
    console.log('使用兜底逻辑');
    for (let i = 0; i < props.lyrics.length; i++) {
      if (currentTimeMs >= props.lyrics[i].time) {
        index = i;
      } else {
        break;
      }
    }
  }

  console.log(`最终选择歌词索引: ${index}`);
  return index >= 0 ? props.lyrics[index] : null;
});

// 添加调试信息
watch(() => props.lyrics, (newLyrics) => {
  console.log('歌词数据更新:', newLyrics);
  if (newLyrics && newLyrics.length > 0) {
    console.log('歌词行数:', newLyrics.length);
    console.log('前几行歌词:', newLyrics.slice(0, 3));
  }
}, { immediate: true });

watch(() => props.currentTime, (newTime) => {
  console.log('当前播放时间:', newTime, '秒');
});

// 添加currentLyricIndex的调试监听
watch(currentLyricIndex, (newIndex, oldIndex) => {
  console.log(`歌词索引变化: ${oldIndex} -> ${newIndex}`);
});

// 监听currentLyric变化的调试
watch(currentLyric, (newLyric, oldLyric) => {
  console.log('当前歌词变化:', {
    old: oldLyric?.text,
    new: newLyric?.text,
    time: newLyric?.time
  });
});

// 监听当前歌词变化，自动滚动到当前行
watch(currentLyric, async (newLyric) => {
  if (!newLyric || !props.lyrics) {
    currentLyricIndex.value = -1;
    return;
  }

  const index = props.lyrics.findIndex(line => line.time === newLyric.time);
  console.log(`设置当前歌词索引: ${index}, 歌词: "${newLyric.text}"`);
  currentLyricIndex.value = index;

  // 等待DOM更新后滚动
  await nextTick();
  scrollToCurrentLyric();
});

// 滚动到当前歌词行
const scrollToCurrentLyric = () => {
  if (!lyricsContainer.value || currentLyricIndex.value === -1) {
    return;
  }

  const currentElement = lyricsContainer.value.querySelector(
    `.lyric-line:nth-child(${currentLyricIndex.value + 1})`
  ) as HTMLElement;

  if (currentElement) {
    const container = lyricsContainer.value;
    const containerHeight = container.clientHeight;
    const elementTop = currentElement.offsetTop;
    const elementHeight = currentElement.clientHeight;

    // 计算滚动位置，让当前歌词居中显示
    const scrollTop = elementTop - containerHeight / 2 + elementHeight / 2;

    container.scrollTo({
      top: scrollTop,
      behavior: 'smooth'
    });
  }
};

// 格式化时间显示
const formatTime = (timeMs: number) => {
  const totalSeconds = Math.floor(timeMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
};

// 点击歌词行跳转到对应时间
const seekToLyric = (line: LyricLine) => {
  const timeInSeconds = Math.floor(line.time / 1000);
  console.log(`点击歌词跳转: "${line.text}" -> ${timeInSeconds}秒`);
  // 发射事件让父组件处理跳转
  emit('seek', timeInSeconds);
};

const emit = defineEmits<{
  seek: [time: number];
}>();
</script>

<template>
  <div class="lyrics-display">
    <!-- 添加调试显示 -->
    <div v-if="lyrics && lyrics.length > 0" class="debug-info" style="font-size: 12px; color: #999; padding: 0.5rem;">
      调试信息: 歌词行数: {{ lyrics.length }}, 当前时间: {{ currentTime }}秒
    </div>
    
    <div v-if="!lyrics || lyrics.length === 0" class="no-lyrics">
      <div class="no-lyrics-icon">🎵</div>
      <p>暂无歌词</p>
      <p style="font-size: 12px; color: #999;">
        调试: lyrics = {{ lyrics }}, length = {{ lyrics?.length }}
      </p>
    </div>
    
    <div v-else ref="lyricsContainer" class="lyrics-container">
      <div
        v-for="(line, index) in lyrics"
        :key="index"
        :class="[
          'lyric-line',
          {
            'current': index === currentLyricIndex,
            'passed': index < currentLyricIndex,
            'upcoming': index > currentLyricIndex
          }
        ]"
        @click="seekToLyric(line)"
      >
        <span class="lyric-time">{{ formatTime(line.time) }}</span>
        <span class="lyric-text">{{ line.text || '♪' }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.lyrics-display {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f9f9f9;
  border-radius: 8px;
  overflow: hidden;
}

.no-lyrics {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
  font-size: 0.9rem;
}

.no-lyrics-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.lyrics-container {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  scroll-behavior: smooth;
}

.lyric-line {
  display: flex;
  align-items: center;
  padding: 0.75rem 0;
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 4px;
  margin: 0.25rem 0;
}

.lyric-line:hover {
  background: rgba(76, 175, 80, 0.1);
}

.lyric-time {
  font-size: 0.8rem;
  color: #999;
  width: 4rem;
  flex-shrink: 0;
  font-family: monospace;
}

.lyric-text {
  flex: 1;
  margin-left: 1rem;
  line-height: 1.5;
  font-size: 0.95rem;
}

/* 当前播放的歌词 */
.lyric-line.current {
  background: rgba(76, 175, 80, 0.15);
  border-left: 3px solid #4caf50;
  padding-left: calc(0.75rem - 3px);
  transform: scale(1.02);
}

.lyric-line.current .lyric-time {
  color: #4caf50;
  font-weight: bold;
}

.lyric-line.current .lyric-text {
  color: #2e7d32;
  font-weight: 500;
}

/* 已经播放过的歌词 */
.lyric-line.passed {
  opacity: 0.6;
}

.lyric-line.passed .lyric-text {
  color: #666;
}

/* 即将播放的歌词 */
.lyric-line.upcoming {
  opacity: 0.8;
}

/* 滚动条样式 */
.lyrics-container::-webkit-scrollbar {
  width: 6px;
}

.lyrics-container::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.lyrics-container::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.lyrics-container::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .lyrics-container {
    padding: 0.5rem;
  }
  
  .lyric-line {
    padding: 0.5rem 0;
  }
  
  .lyric-time {
    width: 3rem;
    font-size: 0.7rem;
  }
  
  .lyric-text {
    font-size: 0.9rem;
    margin-left: 0.5rem;
  }
}
</style>