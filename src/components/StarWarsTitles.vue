<template>
  <div class="star-wars-container">
    <!-- 背景音乐 -->
    <audio
      ref="bgm"
      :src="bgmSrc"
      autoplay
      loop
      style="display:none"
    ></audio>

    <!-- 开场文字 -->
    <p id="start">实现大家听歌欲望的好帮手&hellip;</p>

    <!-- 标题 -->
    <h1>骁天<sub>音乐播放器</sub></h1>

    <!-- 滚动文字容器 -->
    <div id="titles">
      <div id="titlecontent">
        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">很久很久以前</p>

        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">在一个遥远的星系</p>
        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">邪恶的企鹅音乐垄断了市场</p>
        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">它与网抑云，酷猫勾结</p>

        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">导致大家听不了免费的歌</p>

        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">骁天音乐播放器站了出来</p>

        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">成为大家听歌的新希望</p>

        <p style="color:#ff6; font-weight:bold; text-align:center; text-shadow:0 1px 8px #ff6,0 0 2px #fff;">MV 音乐随心听</p>

        <p>    </p>

        <p>All the code is contained in this single HTML file&hellip;</p>

        <p class="center">View the source, Luke!</p>
      
        <p>Sorry. Couldn't resist it.</p>
      
        <p>You're welcome to use this demonstration code in your own sites. Please link back to the original article at:</p>

        <p class="center"><a href="http://www.sitepoint.com/css3-starwars-scrolling-text/">sitepoint.com/<br />css3-starwars-scrolling-text/</a></p>

        <p>and give me a shout on Twitter <a href="https://twitter.com/craigbuckler">@craigbuckler</a> &ndash; I'd love to see how you use and abuse it!</p>

        <p>Finally, Han shot first and the original, unadulterated movies remain the best. Stop fiddling with them, George!</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const bgm = ref<HTMLAudioElement>();
const bgmSrc = '/starwars.mp3'; // 放在 public 目录下

onMounted(() => {
  // 自动播放背景音乐
  if (bgm.value) {
    bgm.value.currentTime = 0;
    bgm.value.play().catch(() => {});
  }
  // 动画结束后自动暂停音乐（建议与App.vue动画时长一致）
  setTimeout(() => {
    if (bgm.value) {
      bgm.value.pause();
      bgm.value.currentTime = 0;
    }
  }, 11000); // 动画时长
});

onUnmounted(() => {
  // 组件卸载时确保音乐停止
  if (bgm.value) {
    bgm.value.pause();
    bgm.value.currentTime = 0;
  }
});

defineOptions({ name: 'StarWarsTitles' });
</script>

<style scoped>
/* 引入字体 */
@import url(https://fonts.googleapis.com/css?family=Droid+Sans:400,700);

/* 新增：全屏动画容器样式 */
.star-wars-container {
  position: fixed;
  z-index: 9999;
  top: 0; left: 0; right: 0; bottom: 0;
  width: 100vw;
  height: 100vh;
  background: #000;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 开场文字样式 */
#start
{
	position: relative;
	width: 16em;
	font-size: 200%;
	font-weight: 400;
	margin: 20% auto;
	color: #4ee;
	opacity: 0;
	z-index: 1;
	animation: intro 2s ease-out;
}

/* 开场文字动画 */
@keyframes intro {
	0% { opacity: 1; }
	90% { opacity: 1; }
	100% { opacity: 0; }
}

/* 主标题样式 */
h1
{
	position: absolute;
	width: 2.6em;
	left: 50%;
	top: 25%;
	font-size: 10em;
	text-align: center;
	margin-left: -1.3em;
	line-height: 0.8em;
	letter-spacing: -0.05em;
	color: #000;
	text-shadow: -2px -2px 0 #ff6, 2px -2px 0 #ff6, -2px 2px 0 #ff6, 2px 2px 0 #ff6;
	opacity: 0;
	z-index: 1;
	animation: logo 5s ease-out 2.5s;
}

h1 sub
{
	display: block;
	font-size: 0.3em;
	letter-spacing: 0;
	line-height: 0.8em;
}

/* 标题动画 */
@keyframes logo {
	0% { transform: scale(1); opacity: 1; }
	50% { opacity: 1; }
	100% { transform: scale(0.1); opacity: 0; }
}

/* 3D滚动效果容器 */
#titles
{
	position: absolute;
	width: 18em;
	height: 50em;
	bottom: 0;
	left: 50%;
	margin-left: -9em;
	font-size: 350%;
	text-align: justify;
	overflow: hidden;
	transform-origin: 50% 100%;
	transform: perspective(300px) rotateX(25deg);
}

/* 渐变遮罩 */
#titles:after
{
	position: absolute;
	content: ' ';
	left: 0;
	right: 0;
	top: 0;
	bottom: 60%;
	background-image: linear-gradient(top, rgba(0,0,0,1) 0%, transparent 100%);
	pointer-events: none;
}

/* 文本样式 */
#titles p
{
	text-align: justify;
	margin: 0.8em 0;
}

#titles p.center
{
	text-align: center;
}

#titles a
{
	color: #ff6;
	text-decoration: underline;
}

/* 滚动内容容器 */
#titlecontent
{
	position: absolute;
	top: 100%;
	animation: scroll 100s linear 4s infinite;
}

/* 滚动动画 */
@keyframes scroll {
	0% { top: 100%; }
	100% { top: -170%; }
}
</style>