use sha1::{Digest, Sha1};

#[allow(unused)]
pub fn get_sha1_by_sort(token: &str, timestamp: &str, nonce: &str, encrypt: &str) -> Option<String> {
    let mut array = vec![token, timestamp, nonce, encrypt];
    // 字符串排序
    array.sort();

    // 连接字符串
    let message = array.join("");
    Some(get_sha1(&message))
}


#[allow(unused)]
pub fn get_sha1(text: &str) -> String {
    // 计算SHA1
    let mut hasher = Sha1::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}


#[cfg(test)]
mod tests {
    use crate::sha1_utils::{get_sha1, get_sha1_by_sort};

    #[test]
    fn test_get_sha1() {
        // String sToken = "QDG6eK";
        // String sCorpID = "wx5823bf96d3bd56c7";
        // String sEncodingAESKey = "jWmYm7qr5nMoAUwZRjGtBxmz3KA1tkAj3ykkR6q2B2C";

        //String sVerifyMsgSig = "5c45ff5e21c57e6ad56bac8758b79b1d9ac89fd3";
        //String sVerifyTimeStamp = "1409659589";
        //String sVerifyNonce = "263014780";
        let sVerifyEchoStr = "P9nAzCzyDtyTWESHep1vC5X9xho/qYX3Zpb4yKa9SKld1DsH3Iyt3tP3zNdtp+4RPcs8TgAE7OaBO+FZXvnaqQ==";

        // 签名 5c45ff5e21c57e6ad56bac8758b79b1d9ac89fd3
        let result = get_sha1_by_sort("QDG6eK", "1409659589", "263014780", sVerifyEchoStr).unwrap();
        assert_eq!(result, "5c45ff5e21c57e6ad56bac8758b79b1d9ac89fd3")
    }
}
