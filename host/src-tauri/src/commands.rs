use std::sync::Mutex;

use tauri::{Emitter, State};

use crate::models::*;
use crate::process;
use crate::state::AppState;

fn emit_log(app: &tauri::AppHandle, message: &str, level: &str) {
    let _ = app.emit("log", LogPayload {
        message: message.to_string(),
        level: level.to_string(),
    });
}

fn emit_progress(
    app: &tauri::AppHandle,
    file_id: u64,
    seg_id: Option<u64>,
    current: u32,
    total: u32,
    status: &FileStatus,
    message: &str,
) {
    let _ = app.emit("progress", ProgressEvent {
        file_id,
        seg_id,
        current,
        total,
        status: status.clone(),
        message: message.to_string(),
    });
}

#[tauri::command]
pub fn add_files(state: State<Mutex<AppState>>, paths: Vec<String>) -> Result<Vec<FileEntry>, String> {
    let mut app = state.lock().map_err(|e| e.to_string())?;
    Ok(app.add_paths(&paths))
}

#[tauri::command]
pub fn remove_files(state: State<Mutex<AppState>>, ids: Vec<u64>) -> Result<(), String> {
    let mut app = state.lock().map_err(|e| e.to_string())?;
    app.remove_files(&ids);
    Ok(())
}

#[tauri::command]
pub fn get_files(state: State<Mutex<AppState>>) -> Result<Vec<FileEntry>, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    Ok(app.get_files())
}

#[tauri::command]
pub fn scan_metadata(
    app_handle: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    ids: Vec<u64>,
) -> Result<(), String> {
    let paths: Vec<(u64, String)> = {
        let mut app = state.lock().map_err(|e| e.to_string())?;
        app.set_active(ids.clone());
        ids.iter()
            .filter_map(|id| {
                let file = app.get_file_mut(*id)?;
                file.status = FileStatus::Running;
                emit_progress(&app_handle, *id, None, 0, 1, &FileStatus::Running, "scanning");
                Some((*id, file.path.clone()))
            })
            .collect()
    };

    for (id, path) in &paths {
        emit_log(&app_handle, &format!("Scanning metadata: {}", path), "info");
        match process::spawn_scanner(path) {
            Ok(results) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(*id) {
                    for r in results {
                        file.metadata = r.metadata;
                    }
                    file.status = FileStatus::Completed;
                }
                emit_progress(&app_handle, *id, None, 1, 1, &FileStatus::Completed, "scan complete");
            }
            Err(e) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(*id) {
                    file.status = FileStatus::Error;
                }
                emit_log(&app_handle, &format!("Scan failed for {}: {}", path, e), "error");
                return Err(e);
            }
        }
    }
    emit_log(&app_handle, "Metadata scan complete", "info");
    Ok(())
}

#[tauri::command]
pub fn transcribe(
    app_handle: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    ids: Vec<u64>,
    model: String,
    lang: String,
) -> Result<(), String> {
    let paths: Vec<(u64, String)> = {
        let mut app = state.lock().map_err(|e| e.to_string())?;
        app.set_active(ids.clone());
        ids.iter()
            .filter_map(|id| {
                let file = app.get_file_mut(*id)?;
                file.status = FileStatus::Running;
                Some((*id, file.path.clone()))
            })
            .collect()
    };

    for (id, path) in &paths {
        emit_log(&app_handle, &format!("Transcribing: {}", path), "info");
        let cancel = state.lock().map_err(|e| e.to_string())?.cancel_flag(*id);
        match process::spawn_transcriber(path, &model, &lang, cancel.unwrap()) {
            Ok(text) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(*id) {
                    file.transcript = Some(text.clone());
                    file.status = FileStatus::Completed;
                }
                emit_log(&app_handle, &format!("Transcribe complete: {}", path), "info");
            }
            Err(e) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(*id) {
                    file.status = FileStatus::Error;
                }
                emit_log(&app_handle, &format!("Transcribe failed for {}: {}", path, e), "error");
                return Err(e);
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn split_video(
    app_handle: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    id: u64,
    segments: Vec<SegmentInput>,
    format: String,
) -> Result<(), String> {
    let video_path = {
        let mut app = state.lock().map_err(|e| e.to_string())?;
        app.set_active(vec![id]);
        let file = app.get_file_mut(id).ok_or("File not found")?;
        file.status = FileStatus::Running;
        file.path.clone()
    };

    emit_log(&app_handle, &format!("Splitting: {}", video_path), "info");
    let cancel = state.lock().map_err(|e| e.to_string())?.cancel_flag(id);
    match process::spawn_splitter(&video_path, &segments, &format, cancel.unwrap()) {
        Ok(result_segments) => {
            let mut app = state.lock().map_err(|e| e.to_string())?;
            if let Some(file) = app.get_file_mut(id) {
                file.segments = result_segments;
                file.status = FileStatus::Completed;
            }
            emit_log(&app_handle, "Split complete", "info");
            Ok(())
        }
        Err(e) => {
            let mut app = state.lock().map_err(|e| e.to_string())?;
            if let Some(file) = app.get_file_mut(id) {
                file.status = FileStatus::Error;
            }
            emit_log(&app_handle, &format!("Split failed: {}", e), "error");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn organize(
    app_handle: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    ids: Vec<u64>,
    template: String,
    dest: String,
    dry_run: bool,
) -> Result<(), String> {
    let source = ".";
    let cancel = state.lock().map_err(|e| e.to_string())?.cancel_flag(ids[0]);
    emit_log(&app_handle, &format!("Organizing files to: {}", dest), "info");
    let result = process::spawn_organizer(source, &dest, &template, dry_run, cancel.unwrap());
    match &result {
        Ok(_) => emit_log(&app_handle, "Organize complete", "info"),
        Err(e) => emit_log(&app_handle, &format!("Organize failed: {}", e), "error"),
    }
    result
}

#[tauri::command]
pub fn execute_pipeline(
    app_handle: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    ids: Vec<u64>,
) -> Result<(), String> {
    let mut success = 0usize;
    let mut failed = 0usize;
    let total = ids.len() as u32;

    emit_log(&app_handle, &format!("Pipeline started for {} files", total), "info");

    for (idx, &id) in ids.iter().enumerate() {
        let path = {
            let mut app = state.lock().map_err(|e| e.to_string())?;
            app.set_active(vec![id]);
            let file = app.get_file_mut(id).ok_or("File not found")?;
            file.status = FileStatus::Running;
            file.path.clone()
        };

        let file_kind = {
            let app = state.lock().map_err(|e| e.to_string())?;
            app.get_files().iter().find(|f| f.id == id).map(|f| f.kind.clone())
        };

        emit_log(&app_handle, &format!("[{}/{}] Processing: {}", idx + 1, total, path), "info");
        let cancel = state.lock().map_err(|e| e.to_string())?.cancel_flag(id);

        if file_kind == Some(FileKind::Video) {
            emit_log(&app_handle, "  -> Extracting audio via splitter...", "info");
            let segments = match process::spawn_splitter(&path, &[], "mp3", cancel.unwrap()) {
                Ok(segs) => segs,
                Err(e) => {
                    let mut app = state.lock().map_err(|e| e.to_string())?;
                    if let Some(file) = app.get_file_mut(id) {
                        file.status = FileStatus::Error;
                    }
                    emit_log(&app_handle, &format!("  -> Split failed: {}", e), "error");
                    failed += 1;
                    continue;
                }
            };
            {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(id) {
                    file.segments = segments;
                }
            }
        }

        let cancel = state.lock().map_err(|e| e.to_string())?.cancel_flag(id);
        emit_log(&app_handle, "  -> Transcribing...", "info");
        match process::spawn_transcriber(&path, "large-v3-turbo", "zh", cancel.unwrap()) {
            Ok(text) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(id) {
                    file.transcript = Some(text);
                    file.status = FileStatus::Completed;
                }
                success += 1;
            }
            Err(e) => {
                let mut app = state.lock().map_err(|e| e.to_string())?;
                if let Some(file) = app.get_file_mut(id) {
                    file.status = FileStatus::Error;
                }
                emit_log(&app_handle, &format!("  -> Transcribe failed: {}", e), "error");
                failed += 1;
            }
        }
    }

    let _ = app_handle.emit("pipeline_done", PipelineDoneEvent { success, failed });
    emit_log(&app_handle, &format!("Pipeline finished: {} succeeded, {} failed", success, failed), "info");

    if failed > 0 {
        Err(format!("{} succeeded, {} failed", success, failed))
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn check_binary(path: String) -> Result<bool, String> {
    use std::path::Path;
    Ok(Path::new(&path).exists())
}

#[tauri::command]
pub fn cancel(state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut app = state.lock().map_err(|e| e.to_string())?;
    app.cancel_all();
    Ok(())
}
