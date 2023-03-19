use thiserror::Error;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Debug, Error)]
pub enum I18nError {
    #[error("source locale not exists")]
    SourceLocaleNotExists,
    #[error("invalid source file format: {0}")]
    InvalidSourceFileFormat(String),
}
