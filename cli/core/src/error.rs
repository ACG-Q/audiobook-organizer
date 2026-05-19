#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("metadata error: {0}")]
    Other(String),
}

#[must_use]
pub fn is_cross_device_error(e: &std::io::Error) -> bool {
    #[cfg(target_os = "linux")]
    {
        matches!(e.raw_os_error(), Some(18)) // EXDEV
    }
    #[cfg(target_os = "macos")]
    {
        matches!(e.raw_os_error(), Some(18)) // EXDEV
    }
    #[cfg(windows)]
    {
        matches!(e.raw_os_error(), Some(17)) // ERROR_NOT_SAME_DEVICE = 0x11
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("template error: {0}")]
    Template(#[from] handlebars::TemplateError),
    #[error("render error: {0}")]
    Render(#[from] handlebars::RenderError),
}

#[derive(Debug, thiserror::Error)]
pub enum WhisperError {
    #[error("model not found")]
    ModelNotFound,
    #[error("inference failed: {0}")]
    InferenceFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
