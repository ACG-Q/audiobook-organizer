# CLI 工具参考

## 环境变量

### AUDIOBOOK_LANG

设置语言，影响所有工具的行为。

```bash
export AUDIOBOOK_LANG=zh   # Linux/macOS
$env:AUDIOBOOK_LANG="zh"   # Windows PowerShell
```

支持的值：`zh`、`ja`、`en` 等 BCP-47 语言标签。

影响范围：
- **core**：模板渲染的日期格式
- **scanner**：元数据编码检测
- **transcriber**：默认识别语言
- **host**：界面语言

## scanner

音频文件元数据扫描器，读取 ID3 等标签信息。

### 用法

```bash
scanner [OPTIONS] <PATH>
```

### 参数

| 参数 | 说明 |
|------|------|
| `PATH` | 要扫描的目录路径 |
| `--stream` | 输出 JSON Lines 格式（供上位机集成） |

### 输出

普通模式：

```
/path/to/file.mp3
  Title:   有声书标题
  Artist:  作者名
  Album:   专辑名
  Duration:3600s
  Bitrate: 128kbps
```

Stream 模式（JSON Lines）：

```jsonl
{"type":"file","path":"/path/to/file.mp3","metadata":{"title":"有声书标题","artist":"作者名","album":"专辑名","duration":3600.0,"bitrate":128000,"sample_rate":44100,"channels":2,"language":"zh"}}
{"type":"done","total":42}
```

## splitter

视频音频提取与分割工具。底层使用 ffmpeg-next 库（libavformat/libavcodec/libswresample），
不依赖外部 ffmpeg/ffprobe 可执行文件。

### 用法

```bash
splitter split [OPTIONS] <VIDEO>
splitter info [OPTIONS] <VIDEO>
```

### 参数

#### split 子命令

| 参数 | 说明 |
|------|------|
| `VIDEO` | 视频文件路径 |
| `--chapters` | 按章节分割 |
| `--segment <START> <END>` | 按时间区间分割（可多次使用） |
| `--chunk-duration <SECS>` | 按固定时长分块（秒） |
| `--format <FMT>` | 输出格式，默认 `mp3`（支持 mp3/wav/flac/m4a/ogg） |
| `--output-dir <DIR>` | 输出目录，默认视频所在目录的 `split/` |
| `--stream` | JSON Lines 输出 |

`--chapters`、`--segment`、`--chunk-duration` 三者互斥，必须指定其一。

`--segment` 的时间格式支持 `HH:MM:SS` 或秒数：

```bash
splitter split video.mp4 --segment 00:00:00 00:30:00 --segment 00:30:00 01:00:00 --format m4a
```

#### info 子命令

| 参数 | 说明 |
|------|------|
| `VIDEO` | 视频文件路径 |
| `--output <FMT>` | 输出格式，`json`（默认）或 `text` |

### 输出

Stream 模式：

```jsonl
{"type":"progress","file":"video.mp4","percent":0.5,"message":"Extracting audio..."}
{"type":"segment","file":"video.mp4","index":0,"path":"/output/chapter_01.mp3"}
{"type":"done","file":"video.mp4","segments":5}
```

## transcriber

Whisper 语音转文字工具。

### 用法

```bash
transcriber transcribe [OPTIONS] <PATH>
transcriber model <COMMAND>
```

### 子命令

#### transcribe

| 参数 | 说明 |
|------|------|
| `PATH` | 音频文件路径 |
| `--model <NAME>` | Whisper 模型名，默认 `large-v3-turbo` |
| `--lang <CODE>` | 语言代码，默认从 `AUDIOBOOK_LANG` 读取 |
| `--stream` | JSON Lines 输出 |

#### model

| 命令 | 说明 |
|------|------|
| `list` | 列出本地已缓存模型 |
| `download <NAME>` | 下载指定模型 |
| `path <NAME>` | 打印指定模型的本地路径 |

### 注意

默认编译不启用 whisper-rs（避免庞大的依赖和 CUDA 工具链），此时 transcriber 输出占位结果：

```bash
cargo build --release -p audiobook-transcriber --features whisper-rs
```

启用后需要系统安装 CUDA（或 CTranslate2 的推理后端）。

## organizer

按 Handlebars 模板重命名和归类音频文件。

### 用法

```bash
organizer [OPTIONS] <SOURCE> <DEST>
```

### 参数

| 参数 | 说明 |
|------|------|
| `SOURCE` | 源目录 |
| `DEST` | 目标目录 |
| `-t, --template <TEMPLATE>` | 文件名模板（必需） |
| `--dry-run` | 预览模式，不实际移动文件 |
| `-j, --threads <N>` | 工作线程数（使用 rayon） |
| `--stream` | JSON Lines 输出 |

### 模板变量

| 变量 | 说明 |
|------|------|
| `{{title}}` | 标题 |
| `{{artist}}` | 作者/艺术家 |
| `{{album}}` | 专辑 |
| `{{date}}` | 日期 |
| `{{track}}` | 音轨号 |
| `{{ext}}` | 文件扩展名 |
| `{{filename}}` | 原始文件名（不含扩展名） |
| `{{path}}` | 原始相对路径 |
| `{{bitrate}}` | 比特率 |
| `{{duration}}` | 时长（秒） |
| `{{sample_rate}}` | 采样率 |
| `{{language}}` | 语言 |
| `{{channels}}` | 声道数 |

### 示例

```bash
# 按 作者/专辑/音轨号-标题.扩展名 组织
organizer ./input ./output --template "{{artist}}/{{album}}/{{track}}-{{title}}.{{ext}}"

# 预览
organizer ./input ./output --template "{{artist}}/{{title}}.{{ext}}" --dry-run
```
