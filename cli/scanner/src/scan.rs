use std::path::Path;

use audiobook_organizer_core::{AudioFile, AudioMetadata};

pub fn scan(path: &Path) -> anyhow::Result<Vec<AudioFile>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(path).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("scan: skipped entry: {e}");
                continue;
            }
        };
        if entry.file_type().is_file() {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(
                ext.as_str(),
                "mp3" | "m4a" | "flac" | "ogg" | "opus" | "wav"
            ) {
                let path = entry.path().to_path_buf();
                match read_metadata(&path) {
                    Ok(metadata) => files.push(AudioFile { path, metadata }),
                    Err(e) => eprintln!("scan: skipping {}: {e}", path.display()),
                }
            }
        }
    }
    Ok(files)
}

pub fn read_metadata(path: &Path) -> anyhow::Result<AudioMetadata> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut meta = AudioMetadata {
        ext: ext.clone(),
        name: stem.clone(),
        stem,
        ..Default::default()
    };

    let file = std::fs::File::open(path)?;
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = symphonia::core::formats::probe::Hint::new();
    hint.with_extension(&ext);

    if let Ok(mut format) =
        symphonia::default::get_probe().probe(&hint, mss, Default::default(), Default::default())
    {
        if let Some(track) = format.tracks().first() {
            let sample_rate = track
                .codec_params
                .as_ref()
                .and_then(|cp| cp.audio())
                .and_then(|a| a.sample_rate)
                .unwrap_or(1);
            meta.duration = track.num_frames.map(|f| f as f64 / sample_rate as f64);
        }
        if let Some(metadata) = format.metadata().current() {
            for tag in &metadata.media.tags {
                let val = match &tag.raw.value {
                    symphonia::core::meta::RawValue::String(s) => s.to_string(),
                    symphonia::core::meta::RawValue::StringList(list) => list.join(", "),
                    symphonia::core::meta::RawValue::UnsignedInt(n) => n.to_string(),
                    symphonia::core::meta::RawValue::SignedInt(n) => n.to_string(),
                    symphonia::core::meta::RawValue::Float(f) => f.to_string(),
                    _ => continue,
                };
                match tag.raw.key.as_str() {
                    "title" => meta.title = Some(val),
                    "artist" => meta.artist = Some(val),
                    "album" => meta.album = Some(val),
                    "track" => {
                        if let Ok(s) = val.parse::<u32>() {
                            meta.track = Some(s);
                        }
                    }
                    "date" => meta.date = Some(val),
                    "genre" => meta.genre = Some(val),
                    "disc" | "disk" => {
                        if let Ok(s) = val.parse::<u32>() {
                            meta.disc = Some(s);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(meta)
}
