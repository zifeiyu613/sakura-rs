use serde::{Deserialize, Serialize};

/// 计算应用数量的辅助宏
macro_rules! count_apps {
    () => { 0 };
    ($head:expr $(, $tail:expr)*) => { 1 + count_apps!($($tail),*) };
}

/// 定义应用枚举及其所有方法的主宏
macro_rules! define_apps {
    ($(($variant:ident, $id:expr, $code:expr, $name:expr)),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx", sqlx(type_name = "app_type"))]
        #[cfg_attr(feature = "sqlx", sqlx(rename_all = "lowercase"))]
        pub enum App {
            $($variant),*
        }

        impl App {
            /// 获取应用唯一编码
            pub fn code(&self) -> &'static str {
                match self {
                    $(Self::$variant => $code),*
                }
            }

            /// 获取应用中文名称
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }

            /// 获取应用ID (数值标识)
            pub fn id(&self) -> u8 {
                match self {
                    $(Self::$variant => $id),*
                }
            }

            /// 从编码字符串转换为枚举
            pub fn from_code(code: &str) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)*
                    _ => None,
                }
            }

            /// 从ID转换为枚举
            pub fn from_id(id: u8) -> Option<Self> {
                match id {
                    $($id => Some(Self::$variant),)*
                    _ => None,
                }
            }

            /// 获取所有应用列表
            pub fn all() -> &'static [App] {
                static APPS: [App; count_apps!($(App::$variant),*)] = [$(App::$variant),*];
                &APPS
            }

            // 获取API基础URL
            // pub fn base_url(&self) -> &'static str {
            //     match self {
            //         $(Self::$variant => $base_url),*
            //     }
            // }

            // 获取API完整路径
            // pub fn api_url(&self, endpoint: &str) -> String {
            //     format!("{}/v1/{}", self.base_url(), endpoint)
            // }
        }

        // 为 String 类型实现 TryFrom，便于数据库转换
        impl TryFrom<String> for App {
            type Error = String;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                Self::from_code(&s).ok_or_else(|| format!("Unknown app code: {}", s))
            }
        }

        // 为 &str 类型实现 TryFrom
        impl TryFrom<&str> for App {
            type Error = String;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                Self::from_code(s).ok_or_else(|| format!("Unknown app code: {}", s))
            }
        }

        // 实现 ToString 特性，便于序列化
        impl std::fmt::Display for App {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.code())
            }
        }

    };
}

// 使用宏定义应用
define_apps!(
    (HuaJian, 1, "huajian", "花间"),
    (HuaYou, 2, "huayou", "花友"),

    (YiCe, 6, "yice", "易测"),
    // 添加新应用只需在此添加一行
);
