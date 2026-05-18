use audiobook_organizer_core::{Transcript, WhisperError};

#[test]
fn test_transcript_default() {
    let t = Transcript::default();
    assert_eq!(t.text, String::new());
    assert!(t.segments.is_empty());
    assert!(t.language.is_none());
}

#[test]
fn test_transcript_construction() {
    let t = Transcript {
        text: "hello world".into(),
        segments: vec![(0.0, 1.5, "hello".into()), (1.5, 2.0, "world".into())],
        language: Some("en".into()),
    };
    assert_eq!(t.text, "hello world");
    assert_eq!(t.segments.len(), 2);
    assert_eq!(t.segments[0].2, "hello");
    assert_eq!(t.segments[1].2, "world");
    assert_eq!(t.language.as_deref(), Some("en"));
}

#[test]
fn test_transcript_clone() {
    let t = Transcript {
        text: "foo".into(),
        segments: vec![(0.0, 1.0, "foo".into())],
        language: Some("zh".into()),
    };
    let cloned = t.clone();
    assert_eq!(cloned.text, t.text);
    assert_eq!(cloned.segments, t.segments);
    assert_eq!(cloned.language, t.language);
}

#[test]
fn test_whisper_error_model_not_found() {
    let err = WhisperError::ModelNotFound;
    assert_eq!(err.to_string(), "model not found");
}

#[test]
fn test_whisper_error_inference_failed() {
    let err = WhisperError::InferenceFailed("boom".into());
    assert!(err.to_string().contains("boom"));
}

#[test]
fn test_whisper_error_io() {
    use std::io;
    let io_err = io::Error::new(io::ErrorKind::NotFound, "missing file");
    let err: WhisperError = io_err.into();
    assert!(err.to_string().contains("missing file"));
}
