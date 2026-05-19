use std::fs;
use std::path::Path;

use audiobook_organizer::organize_files;
use audiobook_organizer_core::{AudioFile, AudioMetadata};
use tempfile::tempdir;

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
    let report = organize_files(files, template, tmp.path(), true, false).unwrap();
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
    let report = organize_files(files, template, &dest_root, false, false).unwrap();
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
    let report = organize_files(files, template, &dest_root, false, false).unwrap();
    assert_eq!(report.success, 1);
}

fn read_meta_for_test(path: &Path) -> AudioMetadata {
    AudioMetadata {
        ext: path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase(),
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        stem: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    }
}
