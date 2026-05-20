use std::io::Write;
use std::path::PathBuf;

use audiobook_organizer_core::i18n::Lang;
use audiobook_organizer_core::model::{list_models, model_path};
use audiobook_organizer_core::run_cli;
use audiobook_organizer_core::stream::Emit;
use audiobook_organizer_core::StreamEvent;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "transcriber")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Transcribe {
        path: PathBuf,
        #[arg(short, long, default_value = "large-v3-turbo")]
        model: String,
        #[arg(short, long, default_value = "zh")]
        lang: String,
        #[arg(long)]
        stream: bool,
    },
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    List,
    Download {
        name: String,
        #[arg(long)]
        stream: bool,
    },
    Path {
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    run_cli!(Cli, translate, |cli: Cli| {
        match cli.command {
            Commands::Transcribe {
                path: _path,
                model: _model,
                lang: _lang,
                stream: _stream,
            } => {
                #[cfg(feature = "whisper-rs")]
                return handle_transcribe(_path, _model, _lang, _stream);
                #[cfg(not(feature = "whisper-rs"))]
                {
                    eprintln!("Transcription requires the whisper-rs feature. Rebuild with --features whisper-rs");
                    std::process::exit(1);
                }
            }
            Commands::Model { command } => handle_model(command),
        }
    })
}

#[cfg(feature = "whisper-rs")]
fn handle_transcribe(
    path: PathBuf,
    model: String,
    lang: String,
    stream: bool,
) -> anyhow::Result<()> {
    let model_dir = dirs::data_dir()
        .unwrap_or_default()
        .join("audiobook-organizer/models");
    let model_path = model_dir.join(&model);

    if stream {
        StreamEvent::Start {
            data: serde_json::json!({
                "file": path.display().to_string(),
                "model": model,
            }),
        }
        .emit();
    }

    let ctx = audiobook_transcriber::WhisperContext::from_path(model_path)?;
    let transcript = audiobook_transcriber::transcribe(&ctx, &path, Some(lang.as_str()))?;

    if stream {
        for (start, end, text) in &transcript.segments {
            StreamEvent::Item {
                data: serde_json::json!({
                    "start": start,
                    "end": end,
                    "text": text,
                }),
            }
            .emit();
        }
        StreamEvent::Done {
            summary: serde_json::json!({
                "text": transcript.text,
                "language": transcript.language,
            }),
        }
        .emit();
    } else {
        println!("{}", serde_json::to_string_pretty(&transcript)?);
    }

    Ok(())
}

fn handle_model(command: ModelCommands) -> anyhow::Result<()> {
    match command {
        ModelCommands::List => {
            let models = list_models()?;
            println!("{}", serde_json::to_string_pretty(&models)?);
        }
        ModelCommands::Download { name, stream } => {
            let dest = model_path(&name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let url = format!(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
                name
            );

            if stream {
                StreamEvent::Start {
                    data: serde_json::json!({
                        "name": name,
                        "url": url,
                    }),
                }
                .emit();
            }

            eprintln!("Downloading {} -> {:?}", url, dest);
            let resp = ureq::get(&url).call()?;
            let total = resp
                .header("Content-Length")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            let mut reader = resp.into_reader();
            let mut out = std::fs::File::create(dest)?;
            let mut downloaded: u64 = 0;
            let mut buf = [0u8; 65536];

            loop {
                let n = std::io::Read::read(&mut reader, &mut buf)?;
                if n == 0 {
                    break;
                }
                out.write_all(&buf[..n])?;
                downloaded += n as u64;

                if stream && total > 0 {
                    StreamEvent::<serde_json::Value>::Progress {
                        current: downloaded as usize,
                        total: total as usize,
                    }
                    .emit();
                }
            }

            if stream {
                StreamEvent::Done {
                    summary: serde_json::json!({}),
                }
                .emit();
            }
        }
        ModelCommands::Path { name } => {
            println!("{}", model_path(&name).display());
        }
    }
    Ok(())
}

fn translate(cmd: clap::Command, lang: &Lang) -> clap::Command {
    match lang {
        Lang::Zh => cmd
            .about("语音转文字 (Whisper)")
            .mut_subcommand("transcribe", |s| {
                s.about("转写音频文件")
                    .mut_arg("path", |a| a.help("音频文件路径"))
                    .mut_arg("model", |a| {
                        a.help("Whisper 模型名称（默认: large-v3-turbo）")
                    })
                    .mut_arg("lang", |a| a.help("语言代码（默认: zh）"))
                    .mut_arg("stream", |a| a.help("启用 JSON Lines 流式输出"))
            })
            .mut_subcommand("model", |s| {
                s.about("管理 Whisper 模型")
                    .mut_subcommand("list", |s| s.about("列出本地已下载的模型"))
                    .mut_subcommand("download", |s| {
                        s.about("从 HuggingFace 下载模型")
                            .mut_arg("name", |a| a.help("模型名称，如 large-v3-turbo"))
                            .mut_arg("stream", |a| a.help("启用 JSON Lines 流式输出下载进度"))
                    })
                    .mut_subcommand("path", |s| {
                        s.about("打印模型的本地路径")
                            .mut_arg("name", |a| a.help("模型名称"))
                    })
            }),
        Lang::En => cmd
            .about("Speech-to-text via Whisper")
            .mut_subcommand("transcribe", |s| {
                s.about("Transcribe an audio file")
                    .mut_arg("path", |a| a.help("Audio file path"))
                    .mut_arg("model", |a| {
                        a.help("Whisper model name (default: large-v3-turbo)")
                    })
                    .mut_arg("lang", |a| a.help("Language code (default: zh)"))
                    .mut_arg("stream", |a| a.help("Enable JSON Lines streaming output"))
            })
            .mut_subcommand("model", |s| {
                s.about("Manage Whisper models")
                    .mut_subcommand("list", |s| s.about("List locally cached models"))
                    .mut_subcommand("download", |s| {
                        s.about("Download model from HuggingFace")
                            .mut_arg("name", |a| a.help("Model name, e.g. large-v3-turbo"))
                            .mut_arg("stream", |a| {
                                a.help("Enable JSON Lines streaming for download progress")
                            })
                    })
                    .mut_subcommand("path", |s| {
                        s.about("Print local path of a model")
                            .mut_arg("name", |a| a.help("Model name"))
                    })
            }),
    }
}
