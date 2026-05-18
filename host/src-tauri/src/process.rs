use std::io::BufRead;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::*;

fn binary_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

fn find_binary(name: &str) -> String {
    let bin_name = binary_name(name);
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join(&bin_name);
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
            if parent.ends_with("debug") {
                if let Some(target_dir) = parent.parent() {
                    let release = target_dir.join("release").join(&bin_name);
                    if release.exists() {
                        return release.to_string_lossy().to_string();
                    }
                }
            }
        }
    }
    bin_name
}

pub fn spawn_scanner(path: &str) -> Result<Vec<FileEntry>, String> {
    let output = Command::new(find_binary("audiobook-scanner"))
        .arg(path)
        .arg("--stream")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to spawn scanner: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("scanner failed: {}", stderr));
    }

    let mut entries = Vec::new();
    for line in output.stdout.lines() {
        let line = line.map_err(|e| format!("read error: {}", e))?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
            if val.get("type").and_then(|v| v.as_str()) == Some("file") {
                let path = val["path"].as_str().unwrap_or("").to_string();
                let meta = val.get("metadata").and_then(|m| {
                    Some(AudioMetadata {
                        title: m.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        artist: m.get("artist").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        album: m.get("album").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        date: m.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        track: m.get("track").map(|v| v.to_string()),
                        duration: m.get("duration").and_then(|v| v.as_f64()),
                    })
                });
                entries.push(FileEntry {
                    id: 0,
                    path,
                    kind: FileKind::Audio,
                    size: 0,
                    status: FileStatus::Completed,
                    metadata: meta,
                    segments: Vec::new(),
                    transcript: None,
                    rename_preview: None,
                    progress: None,
                });
            }
        }
    }
    Ok(entries)
}

pub fn spawn_transcriber(
    path: &str,
    model: &str,
    lang: &str,
    cancel: Arc<AtomicBool>,
) -> Result<String, String> {
    let mut child = Command::new(find_binary("audiobook-transcriber"))
        .arg("transcribe")
        .arg(path)
        .arg("--model")
        .arg(model)
        .arg("--lang")
        .arg(lang)
        .arg("--stream")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn transcriber: {}", e))?;

    let mut transcript = String::new();
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if cancel.load(Ordering::SeqCst) {
                let _ = child.kill();
                return Err("Cancelled".to_string());
            }
            let line = line.map_err(|e| format!("read error: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(text) = val.get("text").and_then(|v| v.as_str()) {
                    transcript.push_str(text);
                    transcript.push(' ');
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("wait error: {}", e))?;
    if !status.success() {
        return Err("transcriber failed".to_string());
    }
    Ok(transcript.trim().to_string())
}

pub fn spawn_splitter(
    video: &str,
    segments: &[SegmentInput],
    format: &str,
    cancel: Arc<AtomicBool>,
) -> Result<Vec<Segment>, String> {
    let mut result_segments = Vec::new();

    for (i, seg) in segments.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err("Cancelled".to_string());
        }

        let output_dir = format!("split_{}", i);
        let output = Command::new(find_binary("audiobook-splitter"))
            .arg("split")
            .arg(video)
            .arg("--segment")
            .arg(seg.start.to_string())
            .arg(seg.end.to_string())
            .arg("--format")
            .arg(format)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--stream")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to spawn splitter: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("splitter 失败: {}", stderr));
        }

        result_segments.push(Segment {
            id: i as u64,
            label: seg.label.clone(),
            start: seg.start,
            end: seg.end,
            path: Some(format!("{}/{}", output_dir, seg.label)),
            status: FileStatus::Completed,
            transcript: None,
            progress: None,
        });
    }

    Ok(result_segments)
}

pub fn spawn_organizer(
    source: &str,
    dest: &str,
    template: &str,
    dry_run: bool,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut cmd = Command::new(find_binary("audiobook-organizer"));
    cmd.arg(source)
        .arg(dest)
        .arg("--template")
        .arg(template)
        .arg("--stream");

    if dry_run {
        cmd.arg("--dry-run");
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn organizer: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if cancel.load(Ordering::SeqCst) {
                let _ = child.kill();
                return Err("Cancelled".to_string());
            }
            let _ = line;
        }
    }

    let status = child.wait().map_err(|e| format!("wait error: {}", e))?;
    if !status.success() {
        return Err("organizer failed".to_string());
    }
    Ok(())
}
