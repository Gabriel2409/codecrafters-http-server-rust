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

    #[error("Can not compress")]
    CanNotCompress,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    StrumParseError(#[from] strum::ParseError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}
pub type Result<T> = core::result::Result<T, Error>;
