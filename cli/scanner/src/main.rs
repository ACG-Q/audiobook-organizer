use std::path::PathBuf;

use audiobook_organizer_core::i18n::Lang;
use audiobook_organizer_core::stream::{Emit, StreamEvent};
use audiobook_organizer_core::run_cli;
use clap::Parser;

mod scan;
use scan::scan;

#[derive(Parser)]
#[command(name = "scanner")]
struct Cli {
    path: PathBuf,
    #[arg(long)]
    stream: bool,
}

fn main() -> anyhow::Result<()> {
    run_cli!(Cli, translate, |cli: Cli| run(cli))
}

fn run(cli: Cli) -> anyhow::Result<()> {
    if cli.stream {
        StreamEvent::<String>::Start { data: cli.path.display().to_string() }.emit();
    }

    let files = scan(&cli.path)?;

    if cli.stream {
        for f in &files {
            StreamEvent::Item { data: f.clone() }.emit();
        }
        StreamEvent::Done { summary: serde_json::json!({"total": files.len()}) }.emit();
    } else {
        println!("{}", serde_json::to_string_pretty(&files)?);
    }

    Ok(())
}

fn translate(cmd: clap::Command, lang: &Lang) -> clap::Command {
    match lang {
        Lang::Zh => cmd
            .about("扫描音频文件并提取元数据")
            .mut_arg("path", |a| a.help("要扫描的目录路径"))
            .mut_arg("stream", |a| a.help("启用 JSON Lines 流式输出，供上位机调用")),
        Lang::En => cmd
            .about("Scan audio files and extract metadata")
            .mut_arg("path", |a| a.help("Directory path to scan"))
            .mut_arg("stream", |a| {
                a.help("Enable JSON Lines streaming output for host PC integration")
            }),
    }
}
