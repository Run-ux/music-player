# 桌面音乐播放器

一个使用 Tauri + Vue + TypeScript + Rust 构建的跨平台桌面音乐播放器应用程序。

## 功能特点

- 支持多种音频格式的播放 (MP3, FLAC, WAV, OGG, M4A, AAC)
- 播放列表管理（添加、删除、清空）
- 播放控制（播放、暂停、停止、下一首、上一首）
- 支持多种播放模式（顺序播放、单曲循环、随机播放）
- 显示歌曲信息（标题、艺术家、专辑）
- 进度条显示和专辑封面展示

## 技术栈

### 前端

- Vue 3 + TypeScript
- Pinia 状态管理
- Tauri API 前后端通信

### 后端

- Rust + Tokio 异步框架
- Rodio 音频播放库
- Symphonia 音频解码
- ID3 标签解析

## 安装与运行

### 环境要求

- [Node.js](https://nodejs.org/) (>= 18.0.0)
- [Rust](https://www.rust-lang.org/) (>= 1.70.0)
- [Tauri 开发环境](https://tauri.app/v1/guides/getting-started/prerequisites)

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在构建时自动安装
```

### 开发模式运行

```bash
npm run tauri dev
```

### 构建发布版本

```bash
npm run tauri build
```
