use std::fs;
use std::path::Path;

use audiobook_organizer_core::{AudioFile, AudioMetadata, RenameReport};
use tempfile::tempdir;

fn organize(
    files: Vec<AudioFile>,
    template_str: &str,
    dest_root: &Path,
    dry_run: bool,
) -> anyhow::Result<RenameReport> {
    let mut report = RenameReport {
        dry_run,
        ..Default::default()
    };
    for file in files {
        let rel = audiobook_organizer_core::render(template_str, &file.metadata)
            .map_err(|e| anyhow::anyhow!("template error: {e}"))?;
        let dest = dest_root.join(&rel);
        if dry_run {
            report.success += 1;
            report.moves.push((file.path, dest));
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::rename(&file.path, &dest) {
            Ok(_) => {
                report.success += 1;
                report.moves.push((file.path, dest));
            }
            Err(e) => {
                let is_cross_device = e.raw_os_error() == Some(17) || e.raw_os_error() == Some(18);
                if is_cross_device {
                    match fs::copy(&file.path, &dest) {
                        Ok(_) => {
                            let _ = fs::remove_file(&file.path);
                            report.success += 1;
                            report.moves.push((file.path, dest));
                        }
                        Err(copy_err) => {
                            report.failed += 1;
                            report.errors
                                .push((file.path, format!("cross-device copy failed: {copy_err}")));
                        }
                    }
                } else {
                    report.failed += 1;
                    report.errors.push((file.path, e.to_string()));
                }
            }
        }
    }
    Ok(report)
}

#[test]
fn test_rename_dry_run() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("a.mp3");
    fs::write(&src, b"fake").unwrap();
    let meta = read_meta_for_test(&src);
    let files = vec![AudioFile {
        path: src.clone(),
        metadata: meta,
    }];
    let template = "{{name}}.{{ext}}";
    let report = organize(files, template, tmp.path(), true).unwrap();
    assert_eq!(report.success, 1);
    assert!(report.dry_run);
    assert_eq!(report.moves.len(), 1);
    assert_eq!(report.failed, 0);
}

#[test]
fn test_rename_creates_dest_and_moves() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("source.mp3");
    fs::write(&src, b"fake audio").unwrap();
    let meta = read_meta_for_test(&src);
    let files = vec![AudioFile {
        path: src.clone(),
        metadata: meta,
    }];
    let template = "{{name}}.{{ext}}";
    let dest_root = tmp.path().join("dest");
    let report = organize(files, template, &dest_root, false).unwrap();
    assert_eq!(report.success, 1);
    assert!(!report.dry_run);
    assert!(dest_root.join("source.mp3").exists());
}

#[test]
fn test_rename_creates_intermediate_dirs() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("a.mp3");
    fs::write(&src, b"fake").unwrap();
    let meta = read_meta_for_test(&src);
    let files = vec![AudioFile {
        path: src.clone(),
        metadata: meta,
    }];
    let dest_root = tmp.path().join("nonexistent_dir").join("subdir");
    let template = "sub/{{name}}.{{ext}}";
    let report = organize(files, template, &dest_root, false).unwrap();
    assert_eq!(report.success, 1);
}

fn read_meta_for_test(path: &Path) -> AudioMetadata {
    AudioMetadata {
        ext: path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase(),
        name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
        stem: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
        ..Default::default()
    }
}
