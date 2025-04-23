

/// 数据库名称 ===> 对应数据库配置
#[derive(Debug)]
pub enum DbName {
    Phoenix,
    Activity,
}

impl DbName {
    pub fn as_str(&self) -> &str {
        match self {
            DbName::Phoenix=> "phoenix",
            DbName::Activity => "activity",
            
        }
    }
}