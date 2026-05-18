#[cfg(feature = "whisper-rs")]
mod transcribe;

#[cfg(feature = "whisper-rs")]
pub use transcribe::{transcribe, WhisperContext};
