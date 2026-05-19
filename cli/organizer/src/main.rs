use std::path::PathBuf;

use audiobook_organizer_core::i18n::Lang;
use audiobook_organizer_core::run_cli;
use audiobook_organizer_core::stream::Emit;
use audiobook_organizer_core::StreamEvent;
use clap::Parser;

use audiobook_organizer::organize_files;

#[derive(Parser)]
#[command(name = "organizer")]
struct Cli {
    source: PathBuf,
    dest: PathBuf,

    #[arg(short, long)]
    template: String,

    #[arg(long)]
    dry_run: bool,

    #[arg(short = 'j', long)]
    threads: Option<usize>,

    #[arg(long)]
    stream: bool,
}

fn main() -> anyhow::Result<()> {
    run_cli!(Cli, translate, |cli: Cli| -> anyhow::Result<()> {
        let files = audiobook_scanner::scan(&cli.source)?;
        let total = files.len();

        if cli.stream {
            StreamEvent::Start {
                data: serde_json::json!({
                    "source": cli.source.display().to_string(),
                    "dest": cli.dest.display().to_string(),
                    "template": cli.template,
                    "total": total,
                }),
            }
            .emit();
        }

        let report = if let Some(n) = cli.threads {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(n).build()?;
            pool.install(
                || organize_files(files, &cli.template, &cli.dest, cli.dry_run, cli.stream),
            )?
        } else {
            organize_files(files, &cli.template, &cli.dest, cli.dry_run, cli.stream)?
        };

        if cli.stream {
            StreamEvent::Done {
                summary: serde_json::json!({
                    "success": report.success,
                    "failed": report.failed,
                    "dry_run": report.dry_run,
                }),
            }
            .emit();
        } else {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }

        Ok(())
    })?;

    Ok(())
}

fn translate(cmd: clap::Command, lang: &Lang) -> clap::Command {
    match lang {
        Lang::Zh => cmd
            .about("按模板重命名和整理音频文件")
            .mut_arg("source", |a| a.help("源目录"))
            .mut_arg("dest", |a| a.help("目标目录"))
            .mut_arg("template", |a| {
                a.help("文件名模板，例如 {{artist}}/{{title}}.{{ext}}")
            })
            .mut_arg("dry_run", |a| {
                a.help("预览模式：显示将要执行的操作但不实际移动")
            })
            .mut_arg("threads", |a| a.help("工作线程数"))
            .mut_arg("stream", |a| a.help("启用 JSON Lines 流式输出，供上位机调用")),
        Lang::En => cmd
            .about("Rename and organize audio files by template")
            .mut_arg("source", |a| a.help("Source directory"))
            .mut_arg("dest", |a| a.help("Destination directory"))
            .mut_arg("template", |a| {
                a.help("Filename template, e.g. {{artist}}/{{title}}.{{ext}}")
            })
            .mut_arg("dry_run", |a| {
                a.help("Preview mode: show what would be done without moving files")
            })
            .mut_arg("threads", |a| a.help("Number of worker threads"))
            .mut_arg("stream", |a| {
                a.help("Enable JSON Lines streaming output for host PC integration")
            }),
    }
}
