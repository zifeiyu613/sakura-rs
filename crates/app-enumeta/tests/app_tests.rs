#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum App {
    HuaJian,  // 花间
    HuaYou,   // 花友
    YiCe,     // 易测
}

impl App {
    /// 获取应用唯一编码
    pub fn code(&self) -> &'static str {
        match self {
            Self::HuaJian => "huajian",
            Self::HuaYou => "huayou",
            Self::YiCe => "yice",
        }
    }

    /// 获取应用中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::HuaJian => "花间",
            Self::HuaYou => "花友",
            Self::YiCe => "易测",
        }
    }

    /// 获取应用ID (数值标识)
    pub fn id(&self) -> u8 {
        match self {
            Self::HuaJian => 1,
            Self::HuaYou => 2,
            Self::YiCe => 3,
        }
    }

    /// 获取所有应用列表
    pub fn all() -> &'static [App] {
        static APPS: [App; 3] = [App::HuaJian, App::HuaYou, App::YiCe];
        &APPS
    }

    /// 从编码字符串转换为枚举
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "huajian" => Some(Self::HuaJian),
            "huayou" => Some(Self::HuaYou),
            "yice" => Some(Self::YiCe),
            _ => None,
        }
    }

    /// 从ID转换为枚举
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::HuaJian),
            2 => Some(Self::HuaYou),
            3 => Some(Self::YiCe),
            _ => None,
        }
    }

    /// 获取API基础URL
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::HuaJian => "https://api.huajian.com",
            Self::HuaYou => "https://api.huayou.com",
            Self::YiCe => "https://api.yice.com",
        }
    }

    /// 获取API完整路径
    pub fn api_url(&self, endpoint: &str) -> String {
        format!("{}/v1/{}", self.base_url(), endpoint)
    }
}


#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn test_app_metadata() {
        let app = App::HuaJian;
        assert_eq!(app.code(), "huajian");
        assert_eq!(app.name(), "花间");
        assert_eq!(app.id(), 1);

        let app = App::HuaYou;
        assert_eq!(app.code(), "huayou");
        assert_eq!(app.name(), "花友");
        assert_eq!(app.id(), 2);
    }

    #[test]
    fn test_app_conversion() {
        // 从编码字符串
        assert_eq!(App::from_code("huajian"), Some(App::HuaJian));
        assert_eq!(App::from_code("invalid"), None);

        // 从ID
        assert_eq!(App::from_id(2), Some(App::HuaYou));
        assert_eq!(App::from_id(99), None);
    }

    #[test]
    fn test_app_all() {
        let all_apps = App::all();
        assert_eq!(all_apps.len(), 3);
        assert!(all_apps.contains(&App::HuaJian));
        assert!(all_apps.contains(&App::HuaYou));
        assert!(all_apps.contains(&App::YiCe));
    }

    #[test]
    fn test_app_urls() {
        let app = App::YiCe;
        assert_eq!(app.base_url(), "https://api.yice.com");
        assert_eq!(app.api_url("users/profile"), "https://api.yice.com/v1/users/profile");
    }

    #[test]
    fn test_app_usage_example() {
        // 遍历所有应用并处理
        for app in App::all() {
            let endpoint = match app {
                App::HuaJian => "flowers",
                App::HuaYou => "friends",
                App::YiCe => "tests",
            };

            let url = app.api_url(endpoint);
            println!("处理应用 {} ({}): API URL = {}", app.name(), app.code(), url);

            // 这里可以添加实际的API调用逻辑
        }
    }
}