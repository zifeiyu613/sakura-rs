

pub enum DatabaseType {
    Phoenix,
    HuajianActivity,
    HuajianLive,
}

impl DatabaseType {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseType::Phoenix => "phoenix",
            DatabaseType::HuajianActivity => "huajian_activity",
            DatabaseType::HuajianLive => "huajian_live",
        }
    }
}
