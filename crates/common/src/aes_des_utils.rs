// use cipher::{Array, BlockCipherDecrypt, BlockCipherEncrypt, KeyInit, KeyIvInit, KeySizeUser, BlockSizeUser};
use aes::cipher::{Array, BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use aes::{Aes128, Aes192, Aes256};
use base64::engine::general_purpose;
use base64::Engine;
use cipher::{typenum, KeySizeUser};
use des::Des;
use std::error::Error;
use block_padding::Pkcs7;

/// AES-128 EBC 加密函数（加密后返回 Base64 编码的字符串）
///
/// # 参数
/// - `key`: 16 字节的密钥（AES-128 要求密钥长度为 16 字节）
/// - `data`: 明文数据
///
/// # 返回
/// 加密后的 Base64 编码字符串，包含 PKCS7 填充
pub fn aes128_encrypt(key: [u8; 16], data: Vec<u8>) -> String {

    let key = Array::from(key);

    // 初始化 AES 加密器
    let cipher = Aes128::new(&key);

    let block_size = 16;
    let padded_data = pkcs7_pad(&data, block_size);

    // 准备存储密文
    let mut encrypted_data = Vec::with_capacity(padded_data.len());

    // 按块加密
    for chunk in padded_data.chunks_exact(block_size) {
        let mut array = [0u8; 16];
        array.copy_from_slice(chunk); // 将块复制到固定大小的数组
        let mut block = Array::from(array);
        cipher.encrypt_block(&mut block);
        encrypted_data.extend_from_slice(block.as_slice());
    }

    encode_base64(encrypted_data).unwrap()
}

/// AES-128 解密函数
///
/// # 参数
/// - `key`: 16 字节的密钥（AES-128 要求密钥长度为 16 字节）
/// - `data`: 密文数据 (经过 base64 编码)
///
/// # 返回
/// 解密后的明文数据
pub fn aes128_decrypt(key: &[u8], data: &str) -> Vec<u8> {
    let data = decode_base64(data).unwrap();
    // Initialize cipher
    let cipher = Aes128::new_from_slice(key).expect("invalid key");

    let mut decrypted_data = Vec::with_capacity(data.len());

    // 遍历每一个 16 字节的块
    for chunk in data.chunks(16) {
        // 确保块长度为16字节
        let mut block = Array::try_from(chunk).expect("invalid data");
        cipher.decrypt_block(&mut block);
        decrypted_data.extend_from_slice(block.as_slice());
    }

    pkcs7_unpad(&decrypted_data, 16).expect("unpad failed")
}

//==================================================================================================


/// AES 加密函数，支持 AES-128 和 AES-256
///
/// # 参数
/// - `key`: 加密密钥（16 字节表示 AES-128，32 字节表示 AES-256）
/// - `data`: 明文数据
///
/// # 返回
/// - Base64 编码的加密密文
pub fn aes_encrypt(key: &[u8], data: &[u8]) -> String {
    // 检测密钥长度，选择加密标准
    match key.len() {
        16 => encrypt_with_cipher::<Aes128>(key, data),
        24 => encrypt_with_cipher::<Aes192>(key, data),
        32 => encrypt_with_cipher::<Aes256>(key, data),
        _ => panic!("Unsupported key length. Use 16 bytes for AES-128 or 32 bytes for AES-256."),
    }
}

/// 使用指定的 AES 密码加密数据
///
/// # 参数
/// - `cipher_type`: AES 密码类型（如 Aes128 或 Aes256）
/// - `key`: 密钥
/// - `data`: 明文数据
///
/// # 返回
/// - Base64 编码的加密密文
fn encrypt_with_cipher<C>(key: &[u8], data: &[u8]) -> String
where
    C: KeyInit + BlockCipherEncrypt,
    <C as KeySizeUser>::KeySize: typenum::Unsigned,
{
    let cipher= C::new_from_slice(key).expect("invalid Key");

    let key_len = key.len();
    // 添加 PKCS7 填充
    let padded_data = pkcs7_pad(data, key_len);

    let mut encrypted_data = Vec::with_capacity(padded_data.len());

    // 按块加密
    for chunk in padded_data.chunks(16) {
        let mut block = Array::try_from(chunk).expect("invalid data");
        cipher.encrypt_block(&mut block);
        encrypted_data.extend_from_slice(block.as_slice());
    }
    // 返回 Base64 编码密文
    encode_base64(encrypted_data).unwrap()
}


/// AES 解密函数，支持 AES-128 和 AES-256
///
/// # 参数
/// - `key`: 加密密钥（16 字节表示 AES-128，32 字节表示 AES-256）
/// - `data`: Base64 编码的密文数据
///
/// # 返回
/// - 解密后的明文数据
///
pub fn aes_decrypt(key: &[u8], data: &str) -> String {
    // 解码 Base64 密文数据
    let encrypted_data = decode_base64(data).expect("Invalid Base64 encoded data");

    // 检测密钥长度，选择加密标准
    match key.len() {
        16 => decrypt_with_cipher::<Aes128>(key, &encrypted_data),
        24 => decrypt_with_cipher::<Aes192>(key, &encrypted_data),
        32 => decrypt_with_cipher::<Aes256>(key, &encrypted_data),
        _ => panic!("Unsupported key length. Use 16 bytes for AES-128，24 bytes for AES-192 or 32 bytes for AES-256."),
    }
}


/// 使用指定的 AES 密码解密数据
///
/// # 参数
/// - `cipher_type`: AES 密码类型（如 Aes128 或 Aes256）
/// - `key`: 密钥
/// - `encrypted_data`: 密文数据
///
/// # 返回
/// - 解密后的明文数据
fn decrypt_with_cipher<C>(key: &[u8], encrypted_data: &[u8]) -> String
where
    C: BlockCipherDecrypt + KeyInit,
    <C as KeySizeUser>::KeySize: typenum::Unsigned,
{
    // 初始化 AES 解密器
    let cipher = C::new_from_slice(key).expect("invalid key");

    let mut decrypted_data = Vec::with_capacity(encrypted_data.len());


    // let mut blocks = encrypted_data.chunks(16).into_iter()
    //     .map(|chunk|Array::from(chunk))
    //     .collect();
    //
    // cipher.decrypt_blocks(&mut blocks);
    //
    // blocks.iter().for_each(|x| {
    //     decrypted_data.extend_from_slice(x.as_slice())
    // });

    // 按块解密
    for chunk in encrypted_data.chunks(16) {
        let mut block = Array::try_from(chunk).expect("invalid data");
        cipher.decrypt_block(&mut block);
        decrypted_data.extend_from_slice(block.as_slice());
    }

    // 去除 PKCS7 填充
    let decrypted_data = pkcs7_unpad(&decrypted_data, key.len()).expect("Invalid padding");
    String::from_utf8_lossy(&decrypted_data).to_string()
}

//==================================================================================================


/// DES 加密函数（加密后返回 Base64 编码的字符串）
///
/// # 参数
/// - `key`: 8 字节的密钥
/// - `iv`: 8 字节的初始化向量
/// - `data`: 明文数据
///
/// # 返回
/// - 加密后的 Base64 编码字符串
pub fn des_encrypt_base64(key: [u8; 8], iv: [u8; 8], data: &str) -> String {
    // 使用 PKCS7 填充
    let padded_data = pkcs7_pad(data.as_bytes(), 8);

    let key = Array::from(key);
    let cipher = Des::new(&key);

    let mut encrypted_data = Vec::with_capacity(padded_data.len());
    let mut current_iv = iv;

    // CBC 模式加密
    for chunk in padded_data.chunks(8) {
        // 与 IV 或前一个密文块异或
        let mut block = [0u8; 8];
        for i in 0..8 {
            block[i] = chunk[i] ^ current_iv[i];
        }

        let mut block = Array::from(block);
        cipher.encrypt_block(&mut block);

        // 更新 IV 为当前密文块
        current_iv.copy_from_slice(block.as_slice());

        encrypted_data.extend_from_slice(block.as_slice());
    }
    // 返回 Base64 编码的密文
    encode_base64(encrypted_data).unwrap()
}

// DES CBC 解密
pub fn des_decrypt(key: [u8; 8], iv: [u8; 8], data: &str) -> Result<String, Box<dyn Error>> {
    let decoded = decode_base64(data)?;
    // 提取 IV 和密文
    if decoded.len() < 8 {
        return Err(Box::from("数据长度不足"));
    }

    // let iv = &decoded[..8];
    let encrypted_data = &decoded;

    let key = Array::from(key);
    let cipher = Des::new(&key);

    let mut decrypted_data = Vec::with_capacity(encrypted_data.len());
    let mut current_iv: [u8; 8] = iv.try_into().unwrap();

    // CBC 模式解密
    for chunk in encrypted_data.chunks(8) {
        let mut block = [0u8; 8];
        block.copy_from_slice(chunk);

        let encrypted_block = Array::from(block);
        let mut decrypted_block = encrypted_block.clone();
        cipher.decrypt_block(&mut decrypted_block);

        // 与 IV 异或
        for i in 0..8 {
            block[i] = decrypted_block[i] ^ current_iv[i];
        }

        // 更新 IV 为上一个密文块
        current_iv.copy_from_slice(chunk);

        decrypted_data.extend_from_slice(&block);
    }

    // 移除 PKCS7 填充
    let decrypted_data =  pkcs7_unpad(&decrypted_data, 8).expect("unpad failed");
    Ok(String::from_utf8_lossy(&decrypted_data).to_string())
}



pub fn des_encrypt_base64_new(key: &[u8], plaintext: &str) -> String {
    // let key = Array::from(key);
    let cipher = Des::new_from_slice(key).expect("invalid key");

    let ciphertext = cipher.encrypt_padded_vec::<Pkcs7>(plaintext.as_bytes());
    // 返回 Base64 编码的密文
    encode_base64(ciphertext).unwrap()
}
pub fn des_decrypt_new(key: &[u8], ciphertext: &str) -> Result<String, Box<dyn Error>> {
    let ciphertext = decode_base64(ciphertext)?;

    let cipher = Des::new_from_slice(key).expect("invalid key");

    let plaintext = cipher.decrypt_padded_vec::<Pkcs7>(&ciphertext)?;

    Ok(String::from_utf8_lossy(&plaintext).to_string())
}

// PKCS7 填充
fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let mut padded = data.to_vec();
    let pad_len = block_size - (data.len() % block_size);
    padded.extend(vec![pad_len as u8; pad_len]);
    padded
}

/// 移除 PKCS7 填充
///
/// # 参数
/// - `data`: 解密后的数据
/// - `block_size`: 块大小
///
/// # 返回
/// - 去填充后的数据，或错误
fn pkcs7_unpad(data: &[u8], block_size: usize) -> Option<Vec<u8>> {
    if let Some(&last_byte) = data.last() {
        let pad_len = last_byte as usize;
        if pad_len > 0 && pad_len <= block_size && data.len() >= pad_len {
            let padding = &data[data.len() - pad_len..];
            if padding.iter().all(|&x| x == last_byte) {
                return Some(data[..data.len() - pad_len].to_vec());
            }
        }
    }
    None
}


/// 从 Base64 字符串解码为二进制数据
///
/// # 参数
/// - `encoded`: Base64 编码的字符串
///
/// # 返回
/// 解码后的二进制数据
pub fn decode_base64(encoded: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let decoded = general_purpose::STANDARD.decode(encoded)?;
    Ok(decoded)
}

/// 从 Base64 字符串解码为二进制数据
///
/// # 参数
/// - `encoded`: Base64 编码的字符串
///
/// # 返回
/// 解码后的二进制数据
pub fn encode_base64(data: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let encoded = general_purpose::STANDARD.encode(data);
    Ok(encoded)
}



// 密钥生成工具
pub struct KeyGenerator;

// impl KeyGenerator {
//     // 生成安全的随机密钥
//     pub fn generate_key() -> [u8; 8] {
//         let mut key = [0u8; 8];
//         rand::thread_rng().fill_bytes(&mut key);
//         key
//     }
//
//     // 从密码短语派生密钥（简单实现）
//     pub fn derive_key_from_passphrase(passphrase: &str) -> [u8; 8] {
//         use sha2::{Sha256, Digest};
//
//         // 使用 SHA-256 哈希密码短语
//         let mut hasher = Sha256::new();
//         hasher.update(passphrase.as_bytes());
//         let hash = hasher.finalize();
//
//         // 取前 8 字节作为密钥
//         hash[..8].try_into().unwrap()
//     }
// }
//
// // 安全的随机 IV 生成
// fn generate_iv() -> [u8; 8] {
//     let mut iv = [0u8; 8];
//     rand::thread_rng().fill_bytes(&mut iv);
//     iv
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn test_aes() {
        let key16 = b"1234567890abcdef"; // 16 字节的密钥
        let key24 = b"1234567890abcdef12345678"; // 24 字节的密钥
        let key32 = b"1234567890abcdef1234567890abcdef"; // 32 字节的密钥
        // 示例数据
        let data = b"Hello, AES Encryption! ssss aaa".to_vec(); // 明文数据

        // 加密数据
        let encrypted_data = aes_encrypt(key16, &data);
        println!("AES128 Encrypted: {:?}", encrypted_data);

        // 解密数据
        let decrypted_data = aes_decrypt(key16, &encrypted_data);
        println!("AES128 Decrypted: {:?}", &decrypted_data);

        println!("AES128 Done ============================");

        // 加密数据
        let encrypted_data = aes_encrypt(key24, &data);
        println!("AES192 Encrypted: {:?}", encrypted_data);

        // 解密数据
        let decrypted_data = aes_decrypt(key24, &encrypted_data);
        println!("AES192 Decrypted: {:?}", &decrypted_data);

        println!("AES192 Done ============================");

        // 加密数据
        let encrypted_data = aes_encrypt(key32, &data);
        println!("AES256 Encrypted: {:?}", encrypted_data);

        // 解密数据
        let decrypted_data = aes_decrypt(key32, &encrypted_data);
        println!("AES256 Decrypted: {:?}", &decrypted_data);

        println!("AES256 Done ============================");
    }

    #[test]
    fn test_des_base64() {
        let key = b"spef11kg"; // 16 字节的密钥

        let iv = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF];
        // 示例数据
        let data = r#"{"tempDTO":{"users":"90003478"},"accountId":"90003948","token":"372e71d5831a8b60a1fece9f4c328ef2","uid":"90003948","version":"59000","source":2,"channel":"huajian","subChannel":"1","device":"vivo-vivo X9"}"#; // 明文数据
        // let data = r#"Hello, DES Encryption!"#; // 明文数据

        let encrypt_data = des_encrypt_base64(*key, iv, data);

        println!("Encrypted: {:?}", encrypt_data);


        let encrypt_data1 = "0OSQhJvlfRmcbqDk2S900CCCg32hO2U+m5Gs3tYEC9ZdgTRTBNbCO8DQLujuQtnJG+3hhfuIkA84CLNPxcvw4g0UEWczPnJBxZkFUtlS+HW/bTXg1zD2xp2UR/5oXkc+3aek0ejN07Oq5J0WESiyl1SBEaPveNKRAIehfkQmb7WZMolwF2bHTUuhAyAC5d085DcXhcnjXEpbJ9hPrvPJcdvs1eLxWGZqc8A59yAxfwVLV/Kp76wALFuipzxy9tfexcNjbYvqaqLBbvH4cvYQtA==";

        assert_eq!(&encrypt_data, encrypt_data1);


        let plaintext = des_decrypt(*key, iv, &encrypt_data).unwrap();


        let ciphertext = des_encrypt_base64_new(key, &plaintext);
        let plaintext1 = des_decrypt_new(key, &ciphertext).unwrap();
        println!("decrypt_data: {:?}", plaintext);
        // println!("decrypt_data1: {:?}", plaintext1);

        assert_eq!(plaintext, plaintext1);

        // println!("encode_base64: {:?}", encode_base64(data.as_bytes().to_vec()).unwrap())

        // String sToken = "QDG6eK";
        // String sCorpID = "wx5823bf96d3bd56c7";
        // String sEncodingAESKey = "jWmYm7qr5nMoAUwZRjGtBxmz3KA1tkAj3ykkR6q2B2C";
        // 解析出url上的参数值如下：
        // String sVerifyMsgSig = HttpUtils.ParseUrl("msg_signature");
        //String sVerifyMsgSig = "5c45ff5e21c57e6ad56bac8758b79b1d9ac89fd3";
        // String sVerifyTimeStamp = HttpUtils.ParseUrl("timestamp");
        //String sVerifyTimeStamp = "1409659589";
        // String sVerifyNonce = HttpUtils.ParseUrl("nonce");
        //String sVerifyNonce = "263014780";
        // String sVerifyEchoStr = HttpUtils.ParseUrl("echostr");
        // String sVerifyEchoStr = "P9nAzCzyDtyTWESHep1vC5X9xho/qYX3Zpb4yKa9SKld1DsH3Iyt3tP3zNdtp+4RPcs8TgAE7OaBO+FZXvnaqQ==";

        // verifyurl echostr: 1616140317555161061


    }
}
