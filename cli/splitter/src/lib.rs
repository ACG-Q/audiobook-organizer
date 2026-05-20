use std::path::{Path, PathBuf};

use audiobook_organizer_core::i18n::{detect_lang, Lang};
use audiobook_organizer_core::stream::{Emit, StreamEvent};
use serde_json::json;

mod encode;

pub fn parse_time(s: &str) -> f64 {
    if let Ok(secs) = s.parse::<f64>() {
        return secs;
    }
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        3 => {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let s: f64 = parts[2].parse().unwrap_or(0.0);
            h * 3600.0 + m * 60.0 + s
        }
        2 => {
            let m: f64 = parts[0].parse().unwrap_or(0.0);
            let s: f64 = parts[1].parse().unwrap_or(0.0);
            m * 60.0 + s
        }
        _ => 0.0,
    }
}

pub fn format_time(secs: f64) -> String {
    let h = (secs as u64) / 3600;
    let m = ((secs as u64) % 3600) / 60;
    let s = secs as u64 % 60;
    let ms = ((secs - secs.floor()) * 1000.0) as u64;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

pub fn run_split(
    video: PathBuf,
    chapters: bool,
    segment: Option<Vec<String>>,
    chunk_duration: Option<f64>,
    format: String,
    output_dir: Option<PathBuf>,
    stream: bool,
) -> anyhow::Result<()> {
    let out_dir = output_dir.unwrap_or_else(|| {
        let parent = video.parent().unwrap_or(Path::new("."));
        parent.join("split")
    });
    std::fs::create_dir_all(&out_dir)?;

    if stream {
        StreamEvent::Start {
            data: json!({
                "video": video.display().to_string(),
                "output_dir": out_dir.display().to_string(),
                "format": format
            }),
        }
        .emit();
    }

    if chapters {
        split_by_chapters(&video, &out_dir, &format, stream)?;
    } else if let Some(seg) = segment {
        if seg.len() == 2 {
            let start = parse_time(&seg[0]);
            let end = parse_time(&seg[1]);
            extract_audio_segment(&video, &out_dir, start, end, &format, stream)?;
        }
    } else if let Some(dur) = chunk_duration {
        split_by_duration(&video, &out_dir, dur, &format, stream)?;
    } else {
        extract_full_audio(&video, &out_dir, &format, stream)?;
    }

    if stream {
        StreamEvent::Done { summary: json!({}) }.emit();
    }

    Ok(())
}

pub fn run_info(video: PathBuf, output: String) -> anyhow::Result<()> {
    let info = get_video_info(&video)?;
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        let zh = matches!(detect_lang(), Lang::Zh);
        if zh {
            println!("文件: {}", video.display());
        } else {
            println!("File: {}", video.display());
        }
        println!(
            "{}: {}s",
            if zh { "时长" } else { "Duration" },
            info["duration"]
        );
        if let Some(chapters) = info["chapters"].as_array() {
            println!(
                "{}: {}",
                if zh { "章节数" } else { "Chapters" },
                chapters.len()
            );
            for ch in chapters {
                println!(
                    "  {}: {} -> {}s",
                    ch["title"].as_str().unwrap_or(""),
                    ch["start"],
                    ch["end"]
                );
            }
        }
    }
    Ok(())
}

fn get_video_info(video: &Path) -> anyhow::Result<serde_json::Value> {
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;

    let file = std::fs::File::open(video)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = video.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let format = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        Default::default(),
    )?;

    let mi = format.media_info();
    let duration = (|| {
        let d = mi.duration?;
        let tb = mi.time_base?;
        Some(d.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64)
    })()
    .unwrap_or(0.0);

    use symphonia::core::meta::ChapterGroupItem;
    fn extract_chapters(items: &[ChapterGroupItem]) -> Vec<serde_json::Value> {
        items
            .iter()
            .flat_map(|item| match item {
                ChapterGroupItem::Chapter(ch) => {
                    let start_sec = ch.start_time.as_secs_f64();
                    let end_sec = ch.end_time.map(|t| t.as_secs_f64()).unwrap_or(0.0);
                    let title = ch
                        .tags
                        .iter()
                        .find(|t| t.raw.key.eq_ignore_ascii_case("title"))
                        .and_then(|t| {
                            if let symphonia::core::meta::RawValue::String(s) = &t.raw.value {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                        .unwrap_or("");
                    vec![json!({
                        "title": title,
                        "start": start_sec,
                        "end": end_sec,
                        "duration": end_sec - start_sec,
                    })]
                }
                ChapterGroupItem::Group(g) => extract_chapters(&g.items),
            })
            .collect()
    }
    let chapters = format
        .chapters()
        .map(|cg| extract_chapters(&cg.items))
        .unwrap_or_default();

    let streams: Vec<serde_json::Value> = format
        .tracks()
        .iter()
        .map(|t| {
            let codec_type = t.codec_params.as_ref().map_or("Unknown".to_string(), |cp| {
                if cp.is_audio() {
                    "Audio".to_string()
                } else if cp.is_video() {
                    "Video".to_string()
                } else {
                    "Subtitle".to_string()
                }
            });
            let codec_name = t
                .codec_params
                .as_ref()
                .map(|cp| codec_id_name(cp))
                .unwrap_or_else(|| "unknown".to_string());
            let sample_rate = t
                .codec_params
                .as_ref()
                .and_then(|cp| cp.audio())
                .and_then(|a| a.sample_rate)
                .unwrap_or(0);
            let ch = t
                .codec_params
                .as_ref()
                .and_then(|cp| cp.audio())
                .and_then(|a| a.channels.as_ref().map(|c| c.count() as u16))
                .unwrap_or(0);
            let duration_str = match t.duration {
                Some(d) => format!("{d}"),
                None => "unknown".to_string(),
            };
            json!({
                "index": t.id,
                "codec_type": codec_type,
                "codec_name": codec_name,
                "time_base": if let Some(tb) = &t.time_base {
                    format!("{}/{}", tb.numer.get(), tb.denom.get())
                } else {
                    "unknown".to_string()
                },
                "start_time": t.start_ts.get(),
                "duration": duration_str,
                "sample_rate": sample_rate,
                "channels": ch,
            })
        })
        .collect();

    Ok(json!({
        "duration": duration,
        "chapters": chapters,
        "streams": streams,
    }))
}

fn codec_id_name(params: &symphonia::core::codecs::CodecParameters) -> String {
    use symphonia::core::codecs::audio::CODEC_ID_NULL_AUDIO;
    if let Some(audio) = params.audio() {
        if audio.codec == CODEC_ID_NULL_AUDIO {
            "unknown".to_string()
        } else {
            format!("{:?}", audio.codec)
        }
    } else {
        "unknown".to_string()
    }
}

fn output_format(output: &Path) -> &str {
    output.extension().and_then(|s| s.to_str()).unwrap_or("wav")
}

fn transcode_segment(
    input: &Path,
    output: &Path,
    start: Option<f64>,
    end: Option<f64>,
) -> anyhow::Result<()> {
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo, TrackType};
    use symphonia::core::io::MediaSourceStream;

    let fmt = output_format(output).to_string();

    let file = std::fs::File::open(input)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = input.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        Default::default(),
    )?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| anyhow::anyhow!("No audio track found"))?;
    let track_id = track.id;

    let audio_params = track
        .codec_params
        .as_ref()
        .and_then(|cp| cp.audio())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No audio codec parameters"))?;
    let sample_rate = audio_params.sample_rate.unwrap_or(44100);

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&audio_params, &AudioDecoderOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create decoder: {e}"))?;

    if let Some(s) = start {
        let time = symphonia::core::units::Time::try_from_secs_f64(s).unwrap_or_default();
        let _ = format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: Some(track_id),
            },
        );
    }

    let mut encoder = encode::create_encoder(output, &fmt)?;
    let start_sample = start.map(|s| (s * sample_rate as f64) as u64).unwrap_or(0);
    let end_sample = end
        .map(|e| (e * sample_rate as f64) as u64)
        .unwrap_or(u64::MAX);
    let mut total_decoded: u64 = 0;
    let mut first_frame = true;

    loop {
        let packet = match format.next_packet() {
            Ok(Some(pkt)) => pkt,
            Ok(None) => break,
            Err(e) => return Err(anyhow::anyhow!("Read error: {e}")),
        };

        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let n_frames = decoded.frames();
                let channels_count = decoded.spec().channels().count();
                let frame_samples = n_frames as u64;

                if n_frames == 0 {
                    continue;
                }

                let mut samples = Vec::with_capacity(n_frames * channels_count);
                decoded.copy_to_vec_interleaved(&mut samples);

                if first_frame {
                    first_frame = false;
                    if start_sample > total_decoded {
                        let skip = (start_sample - total_decoded).min(frame_samples);
                        let keep_start = skip as usize * channels_count;
                        if keep_start < samples.len() {
                            let chunk = &samples[keep_start..];
                            encoder.write_header(sample_rate, channels_count as u16)?;
                            encoder.encode_chunk(chunk)?;
                            total_decoded += frame_samples;
                        } else {
                            total_decoded += frame_samples;
                            continue;
                        }
                    } else {
                        encoder.write_header(sample_rate, channels_count as u16)?;
                        encoder.encode_chunk(&samples)?;
                        total_decoded += frame_samples;
                    }
                } else {
                    if total_decoded + frame_samples > end_sample {
                        let keep = end_sample.saturating_sub(total_decoded) as usize;
                        let keep_end = keep * channels_count;
                        let keep_end = keep_end.min(samples.len());
                        if keep_end > 0 {
                            encoder.encode_chunk(&samples[..keep_end])?;
                        }
                        break;
                    }
                    encoder.encode_chunk(&samples)?;
                    total_decoded += frame_samples;
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => {
                continue;
            }
            Err(e) => return Err(anyhow::anyhow!("Decode error: {e}")),
        }
    }

    encoder.finalize()?;
    Ok(())
}

fn extract_full_audio(
    video: &Path,
    out_dir: &Path,
    format: &str,
    stream: bool,
) -> anyhow::Result<()> {
    let stem = video
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output = out_dir.join(format!("{stem}_full.{format}"));

    if stream {
        StreamEvent::Item {
            data: json!({
                "action": "extracting",
                "source": video.display().to_string(),
                "output": output.display().to_string()
            }),
        }
        .emit();
    }

    transcode_segment(video, &output, None, None)?;

    if stream {
        StreamEvent::<serde_json::Value>::Progress {
            current: 1,
            total: 1,
        }
        .emit();
    }
    Ok(())
}

fn extract_audio_segment(
    video: &Path,
    out_dir: &Path,
    start: f64,
    end: f64,
    format: &str,
    stream: bool,
) -> anyhow::Result<()> {
    let stem = video
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output = out_dir.join(format!(
        "{stem}_{}_{}.{format}",
        format_time(start).replace(':', "."),
        format_time(end).replace(':', ".")
    ));

    if stream {
        StreamEvent::Item {
            data: json!({
                "action": "splitting",
                "segment": "custom",
                "start": start,
                "end": end,
                "output": output.display().to_string()
            }),
        }
        .emit();
    }

    transcode_segment(video, &output, Some(start), Some(end))?;

    if stream {
        StreamEvent::<serde_json::Value>::Progress {
            current: 1,
            total: 1,
        }
        .emit();
    }
    Ok(())
}

fn split_by_chapters(
    video: &Path,
    out_dir: &Path,
    format: &str,
    stream: bool,
) -> anyhow::Result<()> {
    let info = get_video_info(video)?;
    let chapters = info["chapters"].as_array().cloned().unwrap_or_default();
    let total = chapters.len();

    for (i, ch) in chapters.iter().enumerate() {
        let start = ch["start"].as_f64().unwrap_or(0.0);
        let end = ch["end"].as_f64().unwrap_or(0.0);
        let default_title = format!("chapter_{}", i + 1);
        let title = ch["title"].as_str().unwrap_or(&default_title);
        let safe_title: String = title
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let output = out_dir.join(format!("{:02}_{safe_title}.{format}", i + 1));

        if stream {
            StreamEvent::Item {
                data: json!({
                    "action": "splitting",
                    "segment": title,
                    "start": start,
                    "end": end,
                    "output": output.display().to_string()
                }),
            }
            .emit();
        }

        if let Err(e) = transcode_segment(video, &output, Some(start), Some(end)) {
            eprintln!("splitter: failed to extract chapter {i}: {title} — {e}");
        }

        if stream {
            StreamEvent::<serde_json::Value>::Progress {
                current: i + 1,
                total,
            }
            .emit();
        }
    }
    Ok(())
}

fn split_by_duration(
    video: &Path,
    out_dir: &Path,
    chunk_duration: f64,
    format: &str,
    stream: bool,
) -> anyhow::Result<()> {
    let info = get_video_info(video)?;
    let total_duration = info["duration"].as_f64().unwrap_or(0.0);
    let total_chunks = (total_duration / chunk_duration).ceil() as usize;
    let stem = video
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    for i in 0..total_chunks {
        let start = i as f64 * chunk_duration;
        let end = ((i as f64 + 1.0) * chunk_duration).min(total_duration);
        let output = out_dir.join(format!("{stem}_{:02}.{format}", i + 1));

        if stream {
            StreamEvent::Item {
                data: json!({
                    "action": "splitting",
                    "segment": format!("chunk_{}", i + 1),
                    "start": start,
                    "end": end,
                    "output": output.display().to_string()
                }),
            }
            .emit();
        }

        if let Err(e) = transcode_segment(video, &output, Some(start), Some(end)) {
            eprintln!("splitter: failed to extract chunk {i}: {e}");
        }

        if stream {
            StreamEvent::<serde_json::Value>::Progress {
                current: i + 1,
                total: total_chunks,
            }
            .emit();
        }
    }
    Ok(())
}
