use chrono::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThirdPartyError {
    #[error("HTTP request error")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parse error")]
    UrlError(#[from] url::ParseError),

    #[error("Ser parse error")]
    SerError(#[from] serde_urlencoded::ser::Error),

    #[error("Date parse error:{0}")]
    DateParseError(#[from] ParseError),

    #[error("Data parse error:{0}")]
    DataParseError(#[from] serde_json::error::Error),

    #[error("HMAC error")]
    HmacError,

    #[error("Custom error: {0}")]
    Custom(String),
}