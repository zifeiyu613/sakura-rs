#[cfg(test)]
mod tests {
    use serde_json::{from_str, json, to_string};
    use yice_api::utils::app_macro::App;
    // use yice_api::utils::app::App;

    #[test]
    fn test_macro_app_metadata() {
        let app = App::HuaJian;
        assert_eq!(app.code(), "huajian");
        assert_eq!(app.name(), "花间");
        assert_eq!(app.id(), 1);

        let app = App::HuaYou;
        assert_eq!(app.base_url(), "https://api.huayou.com");
    }

    #[test]
    fn test_macro_app_conversion() {
        // 从编码字符串
        assert_eq!(App::from_code("huajian"), Some(App::HuaJian));
        assert_eq!(App::from_code("invalid"), None);

        // 从ID
        assert_eq!(App::from_id(3), Some(App::YiCe));
        assert_eq!(App::from_id(99), None);
    }

    #[test]
    fn test_macro_app_all() {
        let apps = App::all();
        assert_eq!(apps.len(), 3);

        // 验证返回的列表中包含了所有应用
        let app_ids: Vec<u8> = apps.iter().map(|app| app.id()).collect();
        assert_eq!(app_ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_macro_add_new_app() {
        // 此测试展示如何添加新应用的过程
        // 只需在宏调用处添加一行，例如:
        // (XinCe, "xince", "新测", 4, "https://api.xince.com")

        // 然后所有方法都会自动为新应用实现
        // 无需修改任何其他代码
    }

    #[test]
    fn test_macro_usage_example() {
        // 创建应用到处理函数的映射
        let mut processed = vec![];

        // 模拟针对不同应用的业务处理
        for app in App::all() {
            match app {
                App::HuaJian => {
                    let url = app.api_url("flowers/list");
                    println!("处理花间应用: {}", url);
                    processed.push("huajian_processed");
                },
                App::HuaYou => {
                    let url = app.api_url("friends/recommend");
                    println!("处理花友应用: {}", url);
                    processed.push("huayou_processed");
                },
                App::YiCe => {
                    let url = app.api_url("tests/run");
                    println!("处理易测应用: {}", url);
                    processed.push("yice_processed");
                },
            }
        }

        // 验证处理结果
        assert_eq!(processed.len(), 3);
        assert!(processed.contains(&"huajian_processed"));
        assert!(processed.contains(&"huayou_processed"));
        assert!(processed.contains(&"yice_processed"));
    }

    /// 增加 Serde 和 SQLx 相关测试
    #[test]
    fn test_app_serde_serialization() {
        // 序列化
        let app = App::HuaJian;
        let serialized = to_string(&app).unwrap();
        assert_eq!(serialized, "\"huajian\"");

        // 反序列化
        let deserialized: App = from_str("\"huayou\"").unwrap();
        assert_eq!(deserialized, App::HuaYou);

        // 在JSON对象中使用
        let json_obj = json!({
        "app": App::YiCe,
        "data": {
            "user_id": 123
        }
    });
        let serialized = to_string(&json_obj).unwrap();
        assert!(serialized.contains("\"app\":\"yice\""));
    }

    #[test]
    fn test_app_string_conversion() {
        // 从字符串转换
        let app: App = "huajian".try_into().unwrap();
        assert_eq!(app, App::HuaJian);

        // 错误处理
        let result: Result<App, _> = "invalid".try_into();
        assert!(result.is_err());

        // 转换为字符串
        let app_str = App::YiCe.to_string();
        assert_eq!(app_str, "yice");
    }


}