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

## 项目结构

```
music-player/
├── src/                    # 前端 Vue 代码
│   ├── components/         # Vue 组件
│   │   ├── NowPlaying.vue  # 当前播放视图组件
│   │   ├── PlayerControls.vue  # 播放控制组件
│   │   └── Playlist.vue    # 播放列表组件
│   ├── stores/             # Pinia 状态管理
│   │   └── player.ts       # 播放器状态
│   ├── assets/             # 静态资源
│   ├── App.vue             # 主应用组件
│   └── main.ts             # 前端入口
├── src-tauri/              # Rust 后端代码
│   ├── src/                # Rust 源代码
│   │   ├── main.rs         # 应用入口
│   │   ├── lib.rs          # Tauri 命令定义
│   │   └── player.rs       # 音乐播放器实现
│   └── Cargo.toml          # Rust 依赖配置
└── public/                 # 静态资源
    └── default-album.png   # 默认专辑封面
```

## 主要实现细节

### 后端 (Rust)

- **player.rs**:

  - 使用 Rodio 库处理音频播放
  - 实现播放器核心功能和状态管理
  - 通过 ID3 解析音乐元数据

- **lib.rs**:
  - 定义 Tauri 命令接口
  - 处理前后端通信
  - 管理播放器状态

### 前端 (Vue)

- **组件化设计**:

  - 播放控制区域
  - 播放列表管理
  - 当前播放信息展示

- **状态管理**:
  - 使用 Pinia 集中管理状态
  - 响应式更新界面

## 后续扩展方向

1. **界面优化**:

   - 添加主题切换
   - 完善界面动画效果
   - 优化移动端适配

2. **功能扩展**:

   - 添加音频可视化效果
   - 实现歌词显示
   - 添加均衡器功能
   - 支持音乐库管理和自动扫描

3. **性能优化**:
   - 改进大型播放列表处理
   - 优化音频解析性能
   - 减少内存占用

## 如何测试

1. **功能测试**:

   - 测试播放、暂停、停止等控制功能
   - 测试播放列表添加、删除功能
   - 测试不同音频格式的支持

2. **性能测试**:

   - 测试大型播放列表加载
   - 监控内存使用情况
   - 测试长时间运行稳定性

3. **用户体验测试**:
   - 收集用户对界面的反馈
   - 评估操作流程的合理性

## 开发者

此项目由音乐播放器开发团队创建和维护。

## 许可

本项目采用 MIT 许可证。
