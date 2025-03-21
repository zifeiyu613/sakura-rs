use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum NcmFormat {
    MP3,
    FLAC,
}

impl NcmFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            NcmFormat::MP3 => "MP3",
            NcmFormat::FLAC => "FLAC",
        }
    }
}

/// 实现 FromStr 特征
impl FromStr for NcmFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mp3" => Ok(NcmFormat::MP3),
            "flac" => Ok(NcmFormat::FLAC),
            _ => Err(format!("unknown ncm format: {}", s)),
        }
    }
}
