use std::path::PathBuf;

use audiobook_organizer_core::i18n::{detect_lang, Lang};
use audiobook_organizer_core::template;
use clap::{CommandFactory, FromArgMatches, Parser};

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
    let lang = detect_lang();

    let mut cmd = Cli::command();
    cmd = translate(cmd, &lang);
    let matches = cmd
        .try_get_matches_from_mut(std::env::args())
        .unwrap_or_else(|e| e.exit());
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    let files = audiobook_scanner::scan(&cli.source)?;
    let total = files.len();

    if cli.stream {
        let event = serde_json::json!({
            "type":"start",
            "source":cli.source.display().to_string(),
            "dest":cli.dest.display().to_string(),
            "template":cli.template,
            "total":total
        });
        println!("{event}");
    }

    let report = if let Some(n) = cli.threads {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(n).build()?;
        pool.install(|| -> anyhow::Result<_> {
            organize_files(files, &cli.template, &cli.dest, cli.dry_run, cli.stream)
        })?
    } else {
        organize_files(files, &cli.template, &cli.dest, cli.dry_run, cli.stream)?
    };

    if cli.stream {
        let event = serde_json::json!({
            "type":"done",
            "success":report.success,
            "failed":report.failed,
            "dry_run":report.dry_run
        });
        println!("{event}");
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

fn organize_files(
    files: Vec<audiobook_organizer_core::AudioFile>,
    template_str: &str,
    dest_root: &std::path::Path,
    dry_run: bool,
    stream: bool,
) -> anyhow::Result<audiobook_organizer_core::RenameReport> {
    let mut report = audiobook_organizer_core::RenameReport {
        dry_run,
        ..Default::default()
    };

    for (i, file) in files.iter().enumerate() {
        let rel = template::render(template_str, &file.metadata)
            .map_err(|e| anyhow::anyhow!("template error: {e}"))?;
        let dest = dest_root.join(&rel);

        if stream {
            let event = serde_json::json!({
                "type":"organizing",
                "source":file.path.display().to_string(),
                "dest":dest.display().to_string()
            });
            println!("{event}");
        }

        if dry_run {
            report.success += 1;
            report.moves.push((file.path.clone(), dest));
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            match std::fs::rename(&file.path, &dest) {
                Ok(_) => {
                    report.success += 1;
                    report.moves.push((file.path.clone(), dest));
                }
                Err(e) => {
                    let is_cross_device =
                        e.raw_os_error() == Some(17) || e.raw_os_error() == Some(18);
                    if is_cross_device {
                        match std::fs::copy(&file.path, &dest) {
                            Ok(_) => {
                                let _ = std::fs::remove_file(&file.path);
                                report.success += 1;
                                report.moves.push((file.path.clone(), dest));
                            }
                            Err(copy_err) => {
                                report.failed += 1;
                                report.errors.push((
                                    file.path.clone(),
                                    format!("cross-device copy failed: {copy_err}"),
                                ));
                            }
                        }
                    } else {
                        report.failed += 1;
                        report.errors.push((file.path.clone(), e.to_string()));
                    }
                }
            }
        }

        if stream {
            let event = serde_json::json!({
                "type":"progress",
                "current":i + 1,
                "total":files.len()
            });
            println!("{event}");
        }
    }

    Ok(report)
}

fn translate(cmd: clap::Command, lang: &Lang) -> clap::Command {
    match lang {
        Lang::Zh => cmd
            .about("按模板重命名和整理音频文件")
            .mut_arg("source", |a| a.help("源目录"))
            .mut_arg("dest", |a| a.help("目标目录"))
            .mut_arg("template", |a| a.help("文件名模板，例如 {{artist}}/{{title}}.{{ext}}"))
            .mut_arg("dry_run", |a| a.help("预览模式：显示将要执行的操作但不实际移动"))
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
