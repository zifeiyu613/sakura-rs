// use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NcmMetadata {
    #[serde(rename = "album")]
    pub m_album: String,

    #[serde(rename = "artist", deserialize_with = "deserialize_artist")]
    pub m_artist: String,

    #[serde(rename = "format")]
    pub m_format: String,

    #[serde(rename = "albumPic")]
    pub m_album_pic_url: String,

    #[serde(rename = "musicName")]
    pub m_name: String,
    #[serde(rename = "duration")]
    m_duration: u64,
    #[serde(rename = "bitrate")]
    m_bitrate: u64,
}

impl NcmMetadata {
    pub fn new(metadata: &str) -> Self {
        let metadata: NcmMetadata = serde_json::from_str(metadata).unwrap();
        metadata
    }
}

// 为 NcmMetadata 实现 serde::Deserializer trait
// impl<'de> Deserialize<'de> for NcmMetadata {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Deserialize::deserialize(deserializer)
//     }
// }

// 自定义反序列化函数处理 artist 字段, 三种常见实现方式：

// 方法1：直接解析并提取artist名称
fn deserialize_artist<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct ArtistWrapper(Vec<Vec<String>>);

    let wrapper = ArtistWrapper::deserialize(deserializer)?;
    Ok(wrapper
        .0
        .first()
        .and_then(|artist| artist.first())
        .cloned()
        .unwrap_or_default())
}

// fn deserialize_artist_m2<'de, D>(deserializer: D) -> Result<String, D::Error>
// where
//     D: serde::Deserializer<'de>,
// {
//     // 方法2：使用 Visitor 模式
//     struct ArtistVisitor;
//
//     impl<'de> Visitor<'de> for ArtistVisitor {
//         type Value = String;
//
//         fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//             formatter.write_str("an artist name")
//         }
//
//         fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//         where
//             A: SeqAccess<'de>,
//         {
//             // 尝试获取第一个艺术家名称
//             let first_artist: Option<Vec<String>> = seq.next_element()?;
//             Ok(first_artist
//                 .and_then(|artist| artist.first().cloned())
//                 .unwrap_or_default())
//         }
//     }
//
// }

// 方法3：灵活处理多种可能的输入格式
fn deserialize_artist_m3<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ArtistData {
        StringList(Vec<String>),
        NestedList(Vec<Vec<String>>),
    }

    let artist_data = ArtistData::deserialize(deserializer)?;
    match artist_data {
        ArtistData::StringList(list) => Ok(list.first().cloned().unwrap_or_default()),
        ArtistData::NestedList(nested) => Ok(nested
            .first()
            .and_then(|artist| artist.first())
            .cloned()
            .unwrap_or_default()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let metadata = r#"{"musicId":"189602","musicName":"青花","artist":[["周传雄","6652"]],"albumId":"19167","album":"蓝色土耳其","albumPicDocId":"109951169874952327","albumPic":"p4.music.126.net/prDyYpF9EsrLA4O13Tf5hw==/109951169874952327.jpg","bitrate":320000,"mp3DocId":"dd87715e460bb2fa46ea25e1ca79a5f0","duration":297506,"alias":[],"format":"mp3"}"#;

        let ncm_metadata = NcmMetadata::new(metadata);

        assert_eq!(ncm_metadata.m_artist, "周传雄");
    }
}
