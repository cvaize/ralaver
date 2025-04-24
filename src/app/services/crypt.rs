use strum_macros::{Display, EnumString};

#[derive(Debug)]
pub struct CryptService {}

impl CryptService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn encrypt_string(&self, string: &String) -> String{
        "".to_string()
    }

    pub fn decrypt_string(&self, string: &String) -> String{
        "".to_string()
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum CryptServiceError {
    Fail,
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use std::io::{Cursor, Read};
    use base64_stream::ToBase64Reader;
    use base64_stream::FromBase64Reader;
    use openssl::symm::{encrypt, decrypt, Cipher};

    static DATA: &str = "NiEUPdRNJOQhY5WrYvthqHN1IYjBpFlJUR893mYEI1EILn84XU76S9rdD2pIikTv";
    static AES_256_CBC_KEY: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    static AES_256_CBC_IV: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

    #[test]
    fn aes_256_cbc_to_string() {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        let encrypted: Vec<u8> = encrypt(
            cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            data).unwrap();

        let mut reader = ToBase64Reader::new(Cursor::new(encrypted.clone()));

        let mut encrypted_base64 = String::new();

        reader.read_to_string(&mut encrypted_base64).unwrap();

        let mut reader = FromBase64Reader::new(Cursor::new(encrypted_base64.clone()));

        let mut encrypted_after_base64: Vec<u8> = vec![];

        reader.read_to_end(&mut encrypted_after_base64).unwrap();

        assert_eq!(encrypted, encrypted_after_base64);

        let decrypted = decrypt(
            cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            &encrypted_after_base64).unwrap();

        assert_eq!(data, decrypted);
    }

    #[bench]
    fn bench_aes_256_cbc_to_string(b: &mut Bencher) {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();

        b.iter(|| {
            let encrypted: Vec<u8> = encrypt(
                cipher,
                AES_256_CBC_KEY,
                Some(AES_256_CBC_IV),
                data).unwrap();

            let mut reader = ToBase64Reader::new(Cursor::new(encrypted));

            let mut encrypted_base64 = String::new();

            reader.read_to_string(&mut encrypted_base64).unwrap();

            let mut reader = FromBase64Reader::new(Cursor::new(encrypted_base64));

            let mut encrypted_after_base64: Vec<u8> = vec![];

            reader.read_to_end(&mut encrypted_after_base64).unwrap();

            let _ = decrypt(
                cipher,
                AES_256_CBC_KEY,
                Some(AES_256_CBC_IV),
                &encrypted_after_base64).unwrap();
        });
    }

    #[test]
    fn aes_256_cbc() {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        let encrypted = encrypt(
            cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            data).unwrap();

        let decrypted = decrypt(
            cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            &encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[bench]
    fn bench_aes_256_cbc(b: &mut Bencher) {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        b.iter(|| {
            let encrypted = encrypt(
                cipher,
                AES_256_CBC_KEY,
                Some(AES_256_CBC_IV),
                data).unwrap();

            let _ = decrypt(
                cipher,
                AES_256_CBC_KEY,
                Some(AES_256_CBC_IV),
                &encrypted).unwrap();
        });
    }
}
