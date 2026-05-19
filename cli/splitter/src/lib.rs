use std::path::{Path, PathBuf};

use audiobook_organizer_core::i18n::{detect_lang, Lang};
use audiobook_organizer_core::stream::{Emit, StreamEvent};
use serde_json::json;

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
        StreamEvent::Done {
            summary: json!({}),
        }
        .emit();
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

fn init_ffmpeg() {
    use std::sync::OnceLock;
    static INIT: OnceLock<Result<(), ffmpeg_next::Error>> = OnceLock::new();
    let _ = INIT.get_or_init(|| ffmpeg_next::init());
}

fn get_video_info(video: &Path) -> anyhow::Result<serde_json::Value> {
    init_ffmpeg();

    let ictx = ffmpeg_next::format::input(&video)
        .map_err(|e| anyhow::anyhow!("Failed to open video: {e}"))?;

    let duration = ictx.duration() as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64;

    let chapters: Vec<serde_json::Value> = ictx
        .chapters()
        .map(|ch| {
            let tb = ch.time_base();
            let start = ch.start() as f64 * tb.numerator() as f64 / tb.denominator() as f64;
            let end = ch.end() as f64 * tb.numerator() as f64 / tb.denominator() as f64;
            json!({
                "title": ch.metadata().get("title").unwrap_or_default(),
                "start": start,
                "end": end,
                "duration": end - start,
            })
        })
        .collect();

    let streams: Vec<serde_json::Value> = ictx
        .streams()
        .map(|s| {
            let codec =
                ffmpeg_next::codec::context::Context::from_parameters(s.parameters()).ok();
            let codec_name = codec.as_ref().map(|c| c.id().name()).map(|s| s.to_string());
            let codec_long = codec.as_ref().map(|c| format!("{:?}", c.medium()));
            let med_type = format!("{:?}", s.parameters().medium());
            json!({
                "index": s.index(),
                "codec_type": med_type,
                "codec_name": codec_name,
                "codec_long_name": codec_long,
                "time_base": s.time_base().to_string(),
                "start_time": s.start_time(),
                "duration": s.duration(),
                "frames": s.frames(),
            })
        })
        .collect();

    Ok(json!({
        "duration": duration,
        "chapters": chapters,
        "streams": streams,
    }))
}

fn drain_encoder_packets(
    encoder: &mut ffmpeg_next::codec::encoder::audio::Encoder,
    octx: &mut ffmpeg_next::format::context::Output,
    out_tb: ffmpeg_next::Rational,
) -> anyhow::Result<()> {
    let mut out_pkt = ffmpeg_next::packet::Packet::empty();
    loop {
        match encoder.receive_packet(&mut out_pkt) {
            Ok(()) => {
                out_pkt.set_stream(0);
                out_pkt.rescale_ts(encoder.time_base(), out_tb);
                out_pkt.write_interleaved(octx)?;
            }
            Err(ffmpeg_next::Error::Other { errno }) if errno == libc::EAGAIN => {
                return Ok(());
            }
            Err(e) => return Err(anyhow::anyhow!("Encode error: {e}")),
        }
    }
}

fn transcode_segment(
    input: &Path,
    output: &Path,
    start: Option<f64>,
    end: Option<f64>,
) -> anyhow::Result<()> {
    use ffmpeg_next::{codec, encoder, format, frame, media, packet, ChannelLayout};

    init_ffmpeg();

    let mut ictx = format::input(&input)?;
    let input_stream = ictx
        .streams()
        .best(media::Type::Audio)
        .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
    let audio_index = input_stream.index();

    let mut decoder =
        codec::context::Context::from_parameters(input_stream.parameters())?
            .decoder()
            .audio()?;

    let in_tb = input_stream.time_base();

    if let Some(s) = start {
        let seek_ts = (s * ffmpeg_next::ffi::AV_TIME_BASE as f64) as i64;
        let _ = ictx.seek(seek_ts, ..seek_ts);
    }

    let mut octx = format::output(&output)?;

    let codec_id = octx.format().codec(&output, media::Type::Audio);
    let codec = encoder::find(codec_id)
        .ok_or_else(|| anyhow::anyhow!("Encoder not found"))?
        .audio()?;

    let global = octx
        .format()
        .flags()
        .contains(format::flag::Flags::GLOBAL_HEADER);

    let mut ost = octx.add_stream(codec)?;

    {
        let mut enc_ctx = codec::context::Context::from_parameters(ost.parameters())?
            .encoder()
            .audio()?;

        enc_ctx.set_rate(decoder.rate() as i32);

        let ch_layout = codec
            .channel_layouts()
            .map(|cls| cls.best(decoder.channel_layout().channels()))
            .unwrap_or(ChannelLayout::STEREO);
        enc_ctx.set_channel_layout(ch_layout);

        let dst_fmt = codec
            .formats()
            .ok_or_else(|| anyhow::anyhow!("No formats supported by encoder"))?
            .next()
            .unwrap_or(decoder.format());
        enc_ctx.set_format(dst_fmt);

        if global {
            enc_ctx.set_flags(codec::flag::Flags::GLOBAL_HEADER);
        }

        enc_ctx.open_as(codec)?;
        ost.set_parameters(&enc_ctx);
    }

    let mut encoder = codec::context::Context::from_parameters(ost.parameters())?
        .encoder()
        .audio()?;

    let out_tb = ost.time_base();

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header()?;

    for (stream, packet) in ictx.packets() {
        if stream.index() != audio_index {
            continue;
        }

        if let Some(e) = end {
            if let Some(pts) = packet.pts() {
                let t = pts as f64 * in_tb.numerator() as f64 / in_tb.denominator() as f64;
                if t >= e {
                    break;
                }
            }
        }

        let _ = decoder.send_packet(&packet);
        let mut dec_frame = frame::Audio::empty();
        loop {
            match decoder.receive_frame(&mut dec_frame) {
                Ok(()) => {
                    let _ = encoder.send_frame(&dec_frame);
                    drain_encoder_packets(&mut encoder, &mut octx, out_tb)?;
                }
                Err(_) => break,
            }
        }
    }

    let _ = decoder.send_packet(&packet::Packet::empty());
    let mut dec_frame = frame::Audio::empty();
    while decoder.receive_frame(&mut dec_frame).is_ok() {
        let _ = encoder.send_frame(&dec_frame);
        drain_encoder_packets(&mut encoder, &mut octx, out_tb)?;
    }

    let _ = encoder.send_frame(&frame::Audio::empty());
    drain_encoder_packets(&mut encoder, &mut octx, out_tb)?;

    octx.write_trailer()?;
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
        StreamEvent::Progress { current: 1, total: 1 }.emit();
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
        StreamEvent::Progress { current: 1, total: 1 }.emit();
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
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
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
            StreamEvent::Progress {
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
            StreamEvent::Progress {
                current: i + 1,
                total: total_chunks,
            }
            .emit();
        }
    }
    Ok(())
}
