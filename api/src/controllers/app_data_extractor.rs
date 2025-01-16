use serde::{Deserialize, Serialize};
use middleware::RequestContext;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppData {
    pub version: Option<String>,
    pub source: Option<String>,
    pub device: Option<String>,
    pub package_name: Option<String>,
    pub imei: Option<String>,
    pub device_code: Option<String>,
    pub platform: Option<String>,
    pub uid: Option<i64>,
    pub token: Option<String>,
    pub channel: Option<String>,
    pub sub_channel: Option<String>,
    pub network: Option<String>,
}

impl AppData {
    pub fn new(context: &RequestContext) -> AppData {
        match context.form_data.as_ref() {
            Some(form_data) => {
                let data = form_data.data.as_ref();
                if let Some(data) = data {
                    println!("data: {:?}", data);
                    return serde_json::from_str(data).unwrap();
                    // return serde_json::from_str(data.to_string().as_str()).expect("cannot deserialize app data")
                }
                AppData::default()
            },
            None => AppData::default(),
        }
    }
}
