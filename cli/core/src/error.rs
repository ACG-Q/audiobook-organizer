#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("metadata error: {0}")]
    Other(String),
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
