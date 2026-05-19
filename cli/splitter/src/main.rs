use std::path::PathBuf;

use clap::{Parser, Subcommand};

use audiobook_organizer_core::i18n::Lang;

#[derive(Parser)]
#[command(name = "splitter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Split {
        video: PathBuf,

        #[arg(long, conflicts_with_all = &["segment", "chunk_duration"])]
        chapters: bool,

        #[arg(long, num_args = 2, value_names = &["START", "END"])]
        segment: Option<Vec<String>>,

        #[arg(long)]
        chunk_duration: Option<f64>,

        #[arg(long, default_value = "mp3")]
        format: String,

        #[arg(long)]
        output_dir: Option<PathBuf>,

        #[arg(long)]
        stream: bool,
    },
    Info {
        video: PathBuf,
        #[arg(long, default_value = "json")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    audiobook_organizer_core::run_cli!(Cli, translate, |cli: Cli| {
        match cli.command {
            Commands::Split {
                video,
                chapters,
                segment,
                chunk_duration,
                format,
                output_dir,
                stream,
            } => audiobook_splitter::run_split(
                video, chapters, segment, chunk_duration, format, output_dir, stream,
            ),
            Commands::Info { video, output } => {
                audiobook_splitter::run_info(video, output)
            }
        }
    })
}

fn translate(cmd: clap::Command, lang: &Lang) -> clap::Command {
    match lang {
        Lang::Zh => cmd
            .about("从视频中提取音频并拆分")
            .mut_subcommand("split", |s| {
                s.about("提取音频并拆分")
                    .mut_arg("video", |a| a.help("视频文件路径"))
                    .mut_arg("chapters", |a| a.help("按章节拆分"))
                    .mut_arg("segment", |a| {
                        a.value_names(["开始", "结束"])
                            .help("按时间段拆分（格式: HH:MM:SS 或秒数）")
                    })
                    .mut_arg("chunk_duration", |a| {
                        a.help("按固定时长拆分（秒）")
                    })
                    .mut_arg("format", |a| {
                        a.help("输出音频格式（mp3/wav/flac/m4a/ogg）")
                    })
                    .mut_arg("output_dir", |a| {
                        a.help("输出目录（默认: video 同目录下的 split/）")
                    })
                    .mut_arg("stream", |a| a.help("启用 JSON Lines 流式输出"))
            })
            .mut_subcommand("info", |s| {
                s.about("查看视频信息（时长、章节、流）")
                    .mut_arg("video", |a| a.help("视频文件路径"))
                    .mut_arg("output", |a| a.help("输出格式（json/text）"))
            }),
        Lang::En => cmd
            .about("Extract audio from video and split by chapters or time")
            .mut_subcommand("split", |s| {
                s.about("Extract audio and split")
                    .mut_arg("video", |a| a.help("Video file path"))
                    .mut_arg("chapters", |a| {
                        a.help("Split by chapters")
                    })
                    .mut_arg("segment", |a| {
                        a.value_names(["START", "END"])
                            .help("Split by time segment (format: HH:MM:SS or seconds)")
                    })
                    .mut_arg("chunk_duration", |a| {
                        a.help("Split into fixed-duration chunks (seconds)")
                    })
                    .mut_arg("format", |a| {
                        a.help("Output audio format (mp3/wav/flac/m4a/ogg)")
                    })
                    .mut_arg("output_dir", |a| {
                        a.help("Output directory (default: split/ beside the video)")
                    })
                    .mut_arg("stream", |a| {
                        a.help("Enable JSON Lines streaming output for host PC integration")
                    })
            })
            .mut_subcommand("info", |s| {
                s.about("Show video information (duration, chapters, streams)")
                    .mut_arg("video", |a| a.help("Video file path"))
                    .mut_arg("output", |a| a.help("Output format (json/text)"))
            }),
    }
}
