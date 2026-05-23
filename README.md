# 有声书工具集 (Audiobook Organizer)

有声文件批量处理工具集，包含 4 个 CLI 工具和 1 个桌面 GUI（Tauri 上位机）。

## 项目结构

```
audiobook-organizer/
├── cli/                    # CLI 工具
│   ├── core/              共享类型、模板渲染、多语言
│   ├── scanner/           音频文件元数据扫描
│   ├── organizer/         按模板重命名/归类
│   ├── transcriber/       Whisper 语音转文字
│   └── splitter/          音频提取与分割
├── host/                  桌面 GUI（Tauri 上位机）
│   ├── src/               Rust 后端
│   └── frontend/          前端（HTML/CSS/JS）
└── docs/                  技术文档
```

## 前置条件

- Rust 1.75+
- 所有 CLI 二进制文件需在 PATH 中（供上位机调用）

## 构建

```bash
# 构建所有工具
cargo build --release

# 仅构建 CLI 工具
cargo build --release -p audiobook-scanner -p audiobook-organizer -p audiobook-transcriber -p audiobook-splitter
```

## CLI 工具

| 工具 | 用途 | 示例 |
|------|------|------|
| **scanner** | 扫描目录，提取音频元数据 | `scanner ./books --stream` |
| **organizer** | 按模板重命名/归类文件 | `organizer src dest --template "{{artist}}/{{title}}.{{ext}}"` |
| **transcriber** | Whisper 语音转文字 | `transcriber transcribe audio.mp3 --lang zh` |
| **splitter** | 从视频提取音频并按章节/时长分割 | `splitter split video.mp4 --chapters --format mp3` |

所有 CLI 工具支持 `--stream` 参数输出 JSON Lines 格式，供上位机集成。

### 平台支持

| 工具 | Linux | Windows | macOS |
|------|-------|---------|-------|
| scanner | ✅ | ✅ | ✅ |
| organizer | ✅ | ✅ | ✅ |
| splitter | ✅ | ✅ | ✅ |
| transcriber | ✅ | ✅ | ❌\* |

\* `audiobook-transcriber` 在 macOS 目标上跳过编译。`whisper-rs-sys` 内部链接了 `Accelerate.framework`，CI 在 Linux 上交叉编译 macOS 时无法获得此框架。macOS 用户可在本机自行编译（系统自带 Accelerate）。

### AUDIOBOOK_LANG 环境变量

统一通过 `AUDIOBOOK_LANG` 环境变量设置语言（如 `zh`、`ja`、`en`），影响：
- `core` 模板渲染的手册日期格式
- `scanner` 的元数据标签编码检测
- `transcriber` 的默认识别语言
- 上位机的界面语言

## 桌面 GUI（上位机）

Tauri 2.0 桌面应用，提供可视化操作界面：

```
上位机 → scanner → splitter → transcriber → organizer
        (扫描)    (分割)     (转录)       (整理)
```

- 拖拽添加文件/文件夹
- 表格展示文件列表，视频文件可展开显示分割片段
- 流水线一键处理
- 深色主题

```bash
cd host
cargo tauri dev
```

## 许可证

MIT OR Apache-2.0
