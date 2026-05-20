use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileKind {
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Waiting,
    Running,
    Completed,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: u64,
    pub path: String,
    pub kind: FileKind,
    pub size: u64,
    pub status: FileStatus,
    pub metadata: Option<AudioMetadata>,
    pub segments: Vec<Segment>,
    pub transcript: Option<String>,
    pub rename_preview: Option<String>,
    pub progress: Option<ProgressInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub date: Option<String>,
    pub track: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: u64,
    pub label: String,
    pub start: f64,
    pub end: f64,
    pub path: Option<String>,
    pub status: FileStatus,
    pub transcript: Option<String>,
    pub progress: Option<ProgressInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInput {
    pub start: f64,
    pub end: f64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub file_id: u64,
    pub seg_id: Option<u64>,
    pub current: u32,
    pub total: u32,
    pub status: FileStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDoneEvent {
    pub success: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPayload {
    pub message: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeItem {
    pub source: String,
    pub dest: String,
}
