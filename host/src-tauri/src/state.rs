use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use walkdir::WalkDir;

use crate::models::*;

pub struct AppState {
    files: Vec<FileEntry>,
    next_id: u64,
    active_ids: Vec<u64>,
    cancel_flags: HashMap<u64, Arc<AtomicBool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            next_id: 1,
            active_ids: Vec::new(),
            cancel_flags: HashMap::new(),
        }
    }

    pub fn add_paths(&mut self, paths: &[String]) -> Vec<FileEntry> {
        let mut added = Vec::new();
        for p in paths {
            let path = Path::new(p);
            if path.is_dir() {
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.path().is_file() {
                        if let Some(entry) = self.add_single_file(entry.path()) {
                            added.push(entry);
                        }
                    }
                }
            } else if path.is_file() {
                if let Some(entry) = self.add_single_file(path) {
                    added.push(entry);
                }
            }
        }
        added
    }

    fn add_single_file(&mut self, path: &Path) -> Option<FileEntry> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        let audio_exts = ["mp3", "wav", "flac", "m4a", "ogg", "wma", "aac"];
        let video_exts = ["mp4", "mkv", "avi", "mov", "wmv"];

        let kind = if audio_exts.contains(&ext.as_str()) {
            FileKind::Audio
        } else if video_exts.contains(&ext.as_str()) {
            FileKind::Video
        } else {
            return None;
        };

        if self.files.iter().any(|f| f.path == path.to_string_lossy()) {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;

        let metadata = audiobook_scanner::read_metadata(path).ok().map(|m| AudioMetadata {
            title: m.title,
            artist: m.artist,
            album: m.album,
            date: m.date,
            track: m.track.map(|t| t.to_string()),
            duration: m.duration,
        });

        let entry = FileEntry {
            id,
            path: path.to_string_lossy().to_string(),
            kind,
            size: path.metadata().ok().map(|m| m.len()).unwrap_or(0),
            status: FileStatus::Waiting,
            metadata,
            segments: Vec::new(),
            transcript: None,
            rename_preview: None,
            progress: None,
        };

        self.files.push(entry.clone());
        Some(entry)
    }

    pub fn remove_files(&mut self, ids: &[u64]) {
        self.files.retain(|f| !ids.contains(&f.id));
    }

    pub fn get_files(&self) -> Vec<FileEntry> {
        self.files.clone()
    }

    pub fn get_file_mut(&mut self, id: u64) -> Option<&mut FileEntry> {
        self.files.iter_mut().find(|f| f.id == id)
    }

    pub fn get_files_mut(&mut self) -> &mut Vec<FileEntry> {
        &mut self.files
    }

    pub fn set_active(&mut self, ids: Vec<u64>) {
        self.active_ids = ids;
        for &id in &self.active_ids {
            let flag = Arc::new(AtomicBool::new(false));
            self.cancel_flags.insert(id, flag);
        }
    }

    pub fn cancel_flag(&self, id: u64) -> Option<Arc<AtomicBool>> {
        self.cancel_flags.get(&id).cloned()
    }

    pub fn cancel_all(&mut self) {
        for flag in self.cancel_flags.values() {
            flag.store(true, Ordering::SeqCst);
        }
        let ids: Vec<u64> = self.active_ids.drain(..).collect();
        for &id in &ids {
            if let Some(file) = self.get_file_mut(id) {
                file.status = FileStatus::Cancelled;
            }
        }
        self.cancel_flags.clear();
    }

    pub fn has_active(&self) -> bool {
        !self.active_ids.is_empty()
    }
}
