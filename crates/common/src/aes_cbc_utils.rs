use aes::cipher::{block_padding::Pkcs7, BlockModeEncrypt, BlockModeDecrypt, KeyIvInit};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;


type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;


#[allow(unused)]
fn aes_128_cbc_encrypt(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let ct = Aes128CbcEnc::new_from_slices(key, iv).unwrap()
        .encrypt_padded_vec::<Pkcs7>(plaintext);
    ct
}

#[allow(unused)]
fn aes_128_cbc_decrypt(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    Aes128CbcDec::new_from_slices(key, iv).unwrap()
        .decrypt_padded_vec::<Pkcs7>(ciphertext).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_aes_128_cbc() {
        let key16 = b"1234567890abcdef";
        let iv = b"0234567890abcdef";
        let plaintext = b"some plaintext";
        let ciphertext = aes_128_cbc_encrypt(plaintext, key16, iv);

        let plaintext1 = aes_128_cbc_decrypt(ciphertext.as_slice(), key16, iv);

        assert_eq!(plaintext1, b"some plaintext");
    }
}