use audiobook_organizer_core::is_cross_device_error;
use audiobook_organizer_core::stream::Emit;
use audiobook_organizer_core::template;
use audiobook_organizer_core::StreamEvent;

pub fn organize_files(
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
            StreamEvent::Item {
                data: serde_json::json!({
                    "source": file.path.display().to_string(),
                    "dest": dest.display().to_string(),
                }),
            }
            .emit();
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
                Err(e) if is_cross_device_error(&e) => {
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
                }
                Err(e) => {
                    report.failed += 1;
                    report.errors.push((file.path.clone(), e.to_string()));
                }
            }
        }

        if stream {
            StreamEvent::<serde_json::Value>::emit_progress(i + 1, files.len());
        }
    }

    Ok(report)
}
