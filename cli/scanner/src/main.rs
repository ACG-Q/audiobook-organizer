use std::path::PathBuf;

use audiobook_organizer_core::i18n::{detect_lang, Lang};
use clap::{CommandFactory, FromArgMatches, Parser};

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
    let lang = detect_lang();

    let mut cmd = Cli::command();
    cmd = translate(cmd, &lang);
    let matches = cmd
        .try_get_matches_from_mut(std::env::args())
        .unwrap_or_else(|e| e.exit());
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    if cli.stream {
        let event = serde_json::json!({"type":"start","path":cli.path.display().to_string()});
        println!("{event}");
    }

    let files = scan(&cli.path)?;

    if cli.stream {
        for f in &files {
            let event = serde_json::json!({
                "type":"file",
                "path":f.path.display().to_string(),
                "metadata":f.metadata
            });
            println!("{event}");
        }
        let event = serde_json::json!({"type":"done","total":files.len()});
        println!("{event}");
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
