use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentType {
    #[strum(serialize = "APPLE_IAP")]
    AppleIap,
    #[strum(serialize = "WX_SDK")]
    WxSdk,
    #[strum(serialize = "ZFB_SDK")]
    ZfbSdk,
    #[strum(serialize = "WX_H5")]
    WxH5,
    #[strum(serialize = "SHOUFA_WX_H5")]
    ShoufaWxH5,
    #[strum(serialize = "WX_H5_V1")]
    WxH5V1,
    #[strum(serialize = "KJ_WX_H5")]
    KjWxH5,
    #[strum(serialize = "ZFB_H5")]
    ZfbH5,
    #[strum(serialize = "ZFB_H5_V1")]
    ZfbH5V1,
    #[strum(serialize = "ZFB_MIN_PROGRAM")]
    ZfbMinProgram,
    #[strum(serialize = "SHOUFA_ZFB_H5")]
    ShoufaZfbH5,
    #[strum(serialize = "XIAOJU_ZFB_H5")]
    XiaojuZfbH5,
    #[strum(serialize = "ZHILIAN_ALI_H5")]
    ZhilianAliH5,
    #[strum(serialize = "FUBEI_ALI_H5")]
    FubeiAliH5,
    #[strum(serialize = "KUAIJIE_ZFB_H5_V1")]
    KuaijieZfbH5V1,
    #[strum(serialize = "SCAN_PAY_WECHAT")]
    ScanPayWechat,
    #[strum(serialize = "SCAN_KJ_WX")]
    ScanKjWx,
    #[strum(serialize = "SCAN_PAY_ZFB")]
    ScanPayZfb,
    #[strum(serialize = "MIFA_PAY")]
    MifaPay,
    #[strum(serialize = "DY_SDK")]
    DySdk,
    #[strum(serialize = "PAYPAL_H5")]
    PaypalH5,
    #[strum(serialize = "GOOGLE")]
    Google,
    #[strum(serialize = "SD_WX_APPLET_NEW")]
    SdWxAppletNew,
    #[strum(serialize = "SD_ZFB_SDK")]
    SdZfbSdk,
    #[strum(serialize = "QUICK")]
    Quick,
    #[strum(serialize = "SD_H5_APPLET")]
    SdH5Applet,
    #[strum(serialize = "SD_H5_APPLET_JS")]
    SdH5AppletJs,
    #[strum(serialize = "DIN_WX_H5_V2")]
    DinWxH5V2,
    #[strum(serialize = "HEE_PAY")]
    HeePay,
    #[strum(serialize = "HEE_ALI_WAP")]
    HeeAliWap,
    #[strum(serialize = "HLB_ZFB_SDK")]
    HlbZfbSdk,
    #[strum(serialize = "WX_JS")]
    WxJs,
    #[strum(serialize = "SD_WX_JS")]
    SdWxJs,
    #[strum(serialize = "SHOUFA_WX_JS")]
    ShoufaWxJs,
}

impl PaymentType {
    pub fn type_code(&self) -> i32 {
        match self {
            Self::AppleIap => 1,
            Self::WxSdk => 2,
            Self::ZfbSdk => 3,
            Self::WxH5 | Self::ShoufaWxH5 | Self::WxH5V1 | Self::KjWxH5 => 5,
            Self::ZfbH5 | Self::ZfbH5V1 | Self::ZfbMinProgram | Self::ShoufaZfbH5
            | Self::XiaojuZfbH5 | Self::ZhilianAliH5 | Self::FubeiAliH5 | Self::KuaijieZfbH5V1 => 6,
            Self::ScanPayWechat | Self::ScanKjWx | Self::ScanPayZfb => 7,
            Self::MifaPay => 8,
            Self::DySdk => 9,
            Self::PaypalH5 => 18,
            Self::Google => 136,
            Self::SdWxAppletNew => 151,
            Self::SdZfbSdk => 150,
            Self::Quick | Self::SdH5Applet | Self::SdH5AppletJs | Self::DinWxH5V2
            | Self::HeePay | Self::HeeAliWap => 157,
            Self::HlbZfbSdk => 159,
            Self::WxJs | Self::SdWxJs | Self::ShoufaWxJs => 16,
        }
    }

    pub fn sub_type_code(&self) -> i32 {
        match self {
            Self::AppleIap => 1,
            Self::WxSdk => 2,
            Self::ZfbSdk => 3,
            Self::WxH5 => 5,
            Self::ShoufaWxH5 => 135,
            Self::WxH5V1 => 300,
            Self::KjWxH5 => 335,
            Self::ZfbH5 => 6,
            Self::ZfbH5V1 => 301,
            Self::ZfbMinProgram => 302,
            Self::ShoufaZfbH5 => 143,
            Self::XiaojuZfbH5 => 161,
            Self::ZhilianAliH5 => 603,
            Self::FubeiAliH5 => 604,
            Self::KuaijieZfbH5V1 => 605,
            Self::ScanPayWechat => 700,
            Self::ScanKjWx => 701,
            Self::ScanPayZfb => 750,
            Self::MifaPay => 800,
            Self::DySdk => 901,
            Self::PaypalH5 => 18,
            Self::Google => 136,
            Self::SdWxAppletNew => 151,
            Self::SdZfbSdk => 150,
            Self::Quick => 157,
            Self::SdH5Applet => 501,
            Self::SdH5AppletJs => 502,
            Self::DinWxH5V2 => 503,
            Self::HeePay => 153,
            Self::HeeAliWap => 155,
            Self::HlbZfbSdk => 159,
            Self::WxJs => 16,
            Self::SdWxJs => 165,
            Self::ShoufaWxJs => 166,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::AppleIap => "Apple IAP 内购",
            Self::WxSdk => "微信SDK原生支付",
            Self::ZfbSdk => "支付宝SDK原生支付",
            Self::WxH5 => "微信H5支付",
            Self::ShoufaWxH5 => "首发微信H5支付",
            Self::WxH5V1 => "微信H5支付",
            Self::KjWxH5 => "快接微信H5",
            Self::ZfbH5 => "支付宝H5支付",
            Self::ZfbH5V1 => "支付宝H5支付",
            Self::ZfbMinProgram => "支付宝小程序",
            Self::ShoufaZfbH5 => "支付宝H5支付(快接支付V2版本)",
            Self::XiaojuZfbH5 => "小菊支付宝H5支付",
            Self::ZhilianAliH5 => "直连支付支付宝H5",
            Self::FubeiAliH5 => "付呗支付宝H5",
            Self::KuaijieZfbH5V1 => "支付宝H5支付(快接支付V1版本)",
            Self::ScanPayWechat => "扫码-微信支付",
            Self::ScanKjWx => "扫码-快接微信(主扫)",
            Self::ScanPayZfb => "扫码-支付宝支付",
            Self::MifaPay => "跨境支付",
            Self::DySdk => "抖音",
            Self::PaypalH5 => "Paypal H5支付",
            Self::Google => "google支付",
            Self::SdWxAppletNew => "杉德微信小程序New",
            Self::SdZfbSdk => "杉德支付宝SDK",
            Self::Quick => "云闪付",
            Self::SdH5Applet => "杉德支付-H5包装云函数",
            Self::SdH5AppletJs => "杉德支付-H5包装云函数(js插件)",
            Self::DinWxH5V2 => "智付微信H5(V2版本)",
            Self::HeePay => "汇付宝-微信直连",
            Self::HeeAliWap => "汇付宝-支付宝wap",
            Self::HlbZfbSdk => "合利宝支付宝SDK",
            Self::WxJs => "微信公众号支付",
            Self::SdWxJs => "杉德微信公众号支付",
            Self::ShoufaWxJs => "首发(快接)微信公众号支付",
        }
    }

    pub fn from_sub_type(sub_type: i32) -> Option<Self> {
        use strum::IntoEnumIterator;

        Self::iter().find(|p| p.sub_type_code() == sub_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "PROCESSING")]
    Processing,
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "REFUNDED")]
    Refunded,
    #[serde(rename = "PARTIAL_REFUNDED")]
    PartialRefunded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_payment_type_codes() {
        assert_eq!(PaymentType::WxH5.type_code(), 5);
        assert_eq!(PaymentType::WxH5.sub_type_code(), 5);

        assert_eq!(PaymentType::AppleIap.type_code(), 1);
        assert_eq!(PaymentType::AppleIap.sub_type_code(), 1);

        assert_eq!(PaymentType::ZfbH5.type_code(), 6);
        assert_eq!(PaymentType::ZfbH5.sub_type_code(), 6);
    }

    #[test]
    fn test_payment_type_from_sub_type() {
        assert_eq!(PaymentType::from_sub_type(5), Some(PaymentType::WxH5));
        assert_eq!(PaymentType::from_sub_type(2), Some(PaymentType::WxSdk));
        assert_eq!(PaymentType::from_sub_type(999), None);
    }

    #[test]
    fn test_payment_type_description() {
        assert_eq!(PaymentType::WxH5.description(), "微信H5支付");
        assert_eq!(PaymentType::AppleIap.description(), "Apple IAP 内购");
    }

    #[test]
    fn test_payment_type_iteration() {
        let types: Vec<PaymentType> = PaymentType::iter().collect();

        // 确保包含至少一些预期的支付类型
        assert!(types.contains(&PaymentType::WxH5));
        assert!(types.contains(&PaymentType::ZfbH5));
        assert!(types.contains(&PaymentType::AppleIap));
    }
}