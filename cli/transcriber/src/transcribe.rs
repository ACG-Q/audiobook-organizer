use audiobook_organizer_core::{Transcript, WhisperError};

pub type Result<T> = std::result::Result<T, WhisperError>;

#[derive(Debug, Clone)]
pub struct WhisperContext {
    inner: std::sync::Arc<whisper_rs::WhisperContext>,
}

impl WhisperContext {
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            inner: std::sync::Arc::new(whisper_rs::WhisperContext::new(path)?),
        })
    }
}

pub fn transcribe<P: AsRef<std::path::Path>>(
    ctx: &WhisperContext,
    audio_path: P,
    language: Option<&str>,
) -> Result<Transcript> {
    let audio = whisper_rs::WhisperAudioData::read_wav(audio_path)
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    let params = whisper_rs::WhisperContextParameters::default().language(language);

    let mut state = ctx
        .inner
        .create_state()
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    state.set_language(language);

    state
        .full_with_params(audio, None, params)
        .map_err(|e| WhisperError::InferenceFailed(e.to_string()))?;

    let mut segments = Vec::new();
    let mut text = String::new();
    let mut detected_lang = None;

    for segment in state.segments_iter() {
        let start = segment.start();
        let end = segment.end();
        let seg_text = segment.to_string_lossy();
        text.push_str(&seg_text);
        segments.push((start, end, seg_text.to_string()));
    }

    if let Some(lang) = state.language() {
        detected_lang = Some(lang.to_string_lossy());
    }

    Ok(Transcript {
        text,
        segments,
        language: detected_lang,
    })
}
