use std::fs;
use std::path::PathBuf;

use audiobook_organizer_core::{AudioMetadata, RenameReport};
use tempfile::tempdir;

#[test]
fn test_e2e_template_render_roundtrip() {
    let tmp = tempdir().unwrap();
    let f = tmp.path().join("Artist - Title [2024].flac");
    fs::write(&f, b"fake").unwrap();
    let meta = AudioMetadata {
        ext: "flac".into(),
        stem: "Artist - Title [2024]".into(),
        name: "Artist - Title [2024]".into(),
        ..Default::default()
    };
    let out = audiobook_organizer_core::render("{{stem}}.{{ext}}", &meta).unwrap();
    assert!(out.contains("Artist - Title [2024]"));
    assert!(out.contains("flac"));
}

#[test]
fn test_e2e_organize_report_serialization() {
    let report = RenameReport {
        success: 2,
        failed: 0,
        dry_run: true,
        errors: vec![],
        moves: vec![
            (PathBuf::from("a.mp3"), PathBuf::from("out/a.mp3")),
            (PathBuf::from("b.mp3"), PathBuf::from("out/b.mp3")),
        ],
    };
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"success\":2"));
    assert!(json.contains("\"dry_run\":true"));
    assert!(json.contains("a.mp3"));
}
