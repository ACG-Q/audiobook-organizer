use audiobook_organizer_core::{Transcript, WhisperError};
use whisper_rs::{
    convert_integer_to_float_audio, convert_stereo_to_mono_audio, FullParams, SamplingStrategy,
};

pub type Result<T> = std::result::Result<T, WhisperError>;

#[derive(Debug, Clone)]
pub struct WhisperContext {
    inner: std::sync::Arc<whisper_rs::WhisperContext>,
}

impl WhisperContext {
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            inner: std::sync::Arc::new(whisper_rs::WhisperContext::new_with_params(
                path.as_ref(),
                whisper_rs::WhisperContextParameters::default(),
            )?),
        })
    }
}

pub fn transcribe<P: AsRef<std::path::Path>>(
    ctx: &WhisperContext,
    audio_path: P,
    language: Option<&str>,
) -> Result<Transcript> {
    let reader = hound::WavReader::open(audio_path.as_ref())
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;
    let spec = reader.spec();

    if spec.sample_rate != 16000 {
        return Err(WhisperError::InferenceFailed(format!(
            "audio must be 16kHz, got {}Hz",
            spec.sample_rate
        )));
    }

    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    let mut audio = vec![0.0f32; samples.len()];
    convert_integer_to_float_audio(&samples, &mut audio)
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    if spec.channels == 2 {
        audio = convert_stereo_to_mono_audio(&audio)
            .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;
    }

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(language);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let mut state = ctx
        .inner
        .create_state()
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    state
        .full(params, &audio[..])
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    let mut segments = Vec::new();
    let mut text = String::new();

    let num_segments = state
        .full_n_segments()
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    for i in 0..num_segments {
        let seg_text = state
            .full_get_segment_text(i)
            .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;
        let start = state
            .full_get_segment_t0(i)
            .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;
        let end = state
            .full_get_segment_t1(i)
            .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

        text.push_str(&seg_text);
        segments.push((start as f64 / 100.0, end as f64 / 100.0, seg_text));
    }

    Ok(Transcript {
        text,
        segments,
        language: language.map(|s| s.to_string()),
    })
}
