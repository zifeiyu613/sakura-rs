


// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    // #[error("Request error: {0}")]
    // Request(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Download error: {0}")]
    Download(#[from] reqwest::Error),
}