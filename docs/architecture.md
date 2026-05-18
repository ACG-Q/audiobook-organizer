# 系统架构

有声书工具集分为两层：CLI 工具层（独立命令行程序）和 GUI 层（Tauri 上位机）。

## 架构概览

```
┌─────────────────────────────────────────────────┐
│                  上位机 (host/)                   │
│              Tauri 2.0 Desktop GUI               │
│  ┌──────────┐  ┌──────────────┐  ┌────────────┐ │
│  │ 前端      │  │ IPC 命令     │  │ Rust 后端  │ │
│  │ HTML/CSS │←→│ invoke/cancel│←→│ subprocess │ │
│  │ /JS      │  │ /事件        │  │ 管理       │ │
│  └──────────┘  └──────────────┘  └─────┬──────┘ │
└────────────────────────────────────────┼─────────┘
                                         │ spawn
                    ┌────────────────────┼──────────────┐
                    ▼                    ▼              ▼
             ┌──────────┐       ┌────────────┐  ┌────────────┐
             │ scanner  │       │ splitter   │  │ transcriber│
             │ 元数据   │       │ 视频分割   │  │ 语音转录   │
             └──────────┘       └────────────┘  └────────────┘
                    │                                  │
                    └──────────┬───────────────────────┘
                               ▼
                        ┌────────────┐
                        │ organizer  │
                        │ 文件整理   │
                        └────────────┘
```

## 数据流

### 音频文件处理流水线

```
添加文件 → scanner 扫描元数据 → transcriber 语音转文字 → organizer 按模板整理
```

### 视频文件处理流水线

```
添加文件 → scanner 扫描元数据 → splitter 提取音频并分割
       → 每个片段 transcriber 语音转文字 → organizer 按模板整理
```

## CLI 工具层

4 个独立 CLI 二进制程序，使用 `clap` 解析命令行参数。设计原则：

- **独立可执行**：每个工具可单独使用，不依赖其他工具
- **统一接口**：均支持 `--stream` 参数（JSON Lines 格式输出）
- **共享类型**：通过 `core` 库共享数据结构（`AudioMetadata`、模板、i18n）
- **环境变量**：通过 `AUDIOBOOK_LANG` 统一设置语言

### scanner

扫描音频文件并读取元数据标签。支持格式通过 symphonia 实现。

```
用法: scanner <PATH> [--stream]

输出: JSON Lines（--stream）或 格式化文本
      每个文件输出为: {"type":"file","path":"...","metadata":{...}}
      扫描完成输出:   {"type":"done","total":42}
```

### splitter

从视频提取音频并按以下方式之一分割：
- `--chapters`：按章节信息分割
- `--segment <START> <END>`：按指定时间区间分割
- `--chunk-duration <SECS>`：按固定时长分块

底层使用 `ffmpeg-next` 库（libavformat/libavcodec/libswresample）直接读写媒体文件，
不依赖外部 ffmpeg/ffprobe 可执行文件。

```
用法: splitter split <VIDEO> (--chapters | --segment <S> <E> | --chunk-duration <S>) [--format mp3] [--stream]
      splitter info <VIDEO>
```

### transcriber

调用 Whisper 模型进行语音转文字。模型自动从 HuggingFace 下载并缓存。

```
用法: transcriber transcribe <PATH> [--model large-v3-turbo] [--lang zh] [--stream]
      transcriber model list
      transcriber model download <NAME>
      transcriber model path <NAME>
```

默认 feature 不启用 whisper-rs，输出占位结果。启用方式：
```
cargo build --release -p audiobook-transcriber --features whisper-rs
```

### organizer

按 Handlebars 模板将文件从源目录移动到目标目录。

```
用法: organizer <SOURCE> <DEST> --template <TEMPLATE> [--dry-run] [--threads <N>] [--stream]
```

内置模板变量：`title`、`artist`、`album`、`date`、`track`、`ext`、`filename`、`path`、`bitrate`、`duration`、`sample_rate`、`language`、`channels`。

## GUI 层（上位机）

基于 Tauri 2.0 的桌面应用。架构分为三层：

### 前端 (host/frontend/)

纯 HTML/CSS/JavaScript，无框架依赖。

- `index.html`：主界面（文件表格、工具栏、底部状态栏）
- `style.css`：深色主题样式（~11KB）
- `app.js`：状态管理、IPC 调用、事件监听（~16KB）

### 后端 (host/src/)

Rust Tauri 命令 + 状态管理。

- `models.rs`：数据类型（`FileEntry`、`Segment`、`FileKind`、`FileStatus`）
- `state.rs`：应用状态（文件列表、ID 生成、活动进程追踪）
- `process.rs`：子进程管理（spawn scanner/transcriber/splitter/organizer，支持取消）
- `commands.rs`：9 个 IPC 命令注册给前端调用

### 通信

| 方向 | 方式 | 说明 |
|------|------|------|
| 前端→后端 | `invoke()` | 调用 Rust 命令（同步阻塞，Tauri 自动分发到线程池） |
| 后端→前端 | 事件发射 | `progress`、`log`、`pipeline_done`、`segment_added` |
| 取消 | `AtomicBool` | 每个命令创建 `Arc<AtomicBool>`，取消时设置标志，子进程轮询检查 |

## 技术栈

| 层 | 技术 |
|----|------|
| GUI 框架 | Tauri 2.0 |
| 前端 | HTML5 + CSS3 + Vanilla JS |
| 后端 | Rust + serde + walkdir |
| CLI | clap 4 |
| 音频元数据 | symphonia |
| 模板 | handlebars |
| 语音识别 | whisper-rs（可选） |
| 音视频处理 | ffmpeg-next（libavformat/libavcodec/libswresample） |
