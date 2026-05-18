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
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
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
    let mut hint = symphonia::core::probe::Hint::new();
    hint.with_extension(&ext);
    let format_opts = symphonia::core::formats::FormatOptions::default();
    let meta_opts = symphonia::core::meta::MetadataOptions::default();

    if let Ok(probed) = symphonia::default::get_probe().format(&hint, mss, &format_opts, &meta_opts)
    {
        let mut format = probed.format;
        if let Some(track) = format.tracks().first() {
            let codec_params = &track.codec_params;
            meta.duration = codec_params
                .n_frames
                .map(|f| f as f64 / codec_params.sample_rate.unwrap_or(1) as f64);
        }
        if let Some(metadata) = format.metadata().current() {
            for tag in metadata.tags() {
                match tag.key.as_str() {
                    "title" => meta.title = Some(tag.value.to_string()),
                    "artist" => meta.artist = Some(tag.value.to_string()),
                    "album" => meta.album = Some(tag.value.to_string()),
                    "track" => {
                        if let Ok(s) = tag.value.to_string().parse::<u32>() {
                            meta.track = Some(s);
                        }
                    }
                    "date" => meta.date = Some(tag.value.to_string()),
                    "genre" => meta.genre = Some(tag.value.to_string()),
                    "disc" | "disk" => {
                        if let Ok(s) = tag.value.to_string().parse::<u32>() {
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
