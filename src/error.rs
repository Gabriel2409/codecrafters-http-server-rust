use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid request line, expected method, path and http version, got {0}")]
    InvalidRequestLine(String),

    #[error("Invalid request line, missing CRLF")]
    MissingCRLFFromLine,

    #[error("Invalid http header")]
    InvalidHeader,

    #[error("Invalid pool size")]
    InvalidPoolSize,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    StrumParseError(#[from] strum::ParseError),
}
pub type Result<T> = core::result::Result<T, Error>;
