use crate::log_map_err;
use base64_stream::FromBase64Reader;
use base64_stream::ToBase64Reader;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::io::{Cursor, Read};
use strum_macros::{Display, EnumString};

static AES_256_CBC_KEY: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
static AES_256_CBC_IV: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

pub struct CryptService {
    cipher: Cipher,
}

impl CryptService {
    pub fn new() -> Self {
        Self {
            cipher: Cipher::aes_256_cbc(),
        }
    }

    pub fn encrypt_string(&self, string: &str) -> Result<String, CryptServiceError> {
        let encrypted: Vec<u8> = encrypt(
            self.cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            string.as_bytes(),
        )
        .map_err(log_map_err!(
            CryptServiceError::Fail,
            "CryptServiceError::encrypt_string"
        ))?;

        let mut reader = ToBase64Reader::new(Cursor::new(encrypted));

        let mut encrypted_base64 = String::new();
        reader
            .read_to_string(&mut encrypted_base64)
            .map_err(log_map_err!(
                CryptServiceError::Fail,
                "CryptServiceError::encrypt_string"
            ))?;

        Ok(encrypted_base64)
    }

    pub fn decrypt_string(&self, string: &str) -> Result<String, CryptServiceError> {
        let mut reader = FromBase64Reader::new(Cursor::new(string));

        let mut encrypted_after_base64: Vec<u8> = vec![];

        reader
            .read_to_end(&mut encrypted_after_base64)
            .map_err(log_map_err!(
                CryptServiceError::Fail,
                "CryptServiceError::decrypt_string"
            ))?;

        let decrypted = decrypt(
            self.cipher,
            AES_256_CBC_KEY,
            Some(AES_256_CBC_IV),
            &encrypted_after_base64,
        )
        .map_err(log_map_err!(
            CryptServiceError::Fail,
            "CryptServiceError::decrypt_string"
        ))?;

        String::from_utf8(decrypted).map_err(log_map_err!(
            CryptServiceError::Fail,
            "CryptServiceError::decrypt_string"
        ))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum CryptServiceError {
    Fail,
}

#[cfg(test)]
mod tests {
    use crate::CryptService;
    use base64_stream::FromBase64Reader;
    use base64_stream::ToBase64Reader;
    use openssl::symm::{decrypt, encrypt, Cipher};
    use std::io::{Cursor, Read};
    use test::Bencher;

    static DATA: &str =
        "1-10459396685910126978-DLum2QqN6bjg8L7kfMrORdazvv4dlrOT0Z9XcEDZMJ5DAnISYZx18wTHvNI5mlH2";
    static ENCODED_DATA: &str = "7sESJ6EyfRIaUEKTT8NZdy4UVfGziyHa3wSp0/cKlNMvoN0mZzGQhAtpv26CKPTsCHEv+WO2YnxQ3+eMf0FSEN4OCUn+hY9Bo0CE2qZOqdDaPXFwAvO8V8tIehlUdLmE";
    static AES_256_CBC_KEY: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    static AES_256_CBC_IV: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

    // TODO: https://security.stackexchange.com/questions/35210/encrypting-using-aes-256-do-i-need-iv
    // Если вы используете каждый ключ только один раз, то не используйте IV. Если вы используете ключ несколько раз, вам следует использовать каждый раз другой IV, чтобы пара (ключ, IV) не использовалась повторно.
    //
    // Точные требования к IV зависят от выбранного режима цепочки, но обычно достаточно случайного 128-битного значения. Оно должно быть разным для каждого сообщения, которое вы шифруете. Сохраните его вместе с зашифрованным текстом, обычно в качестве префикса.

    #[test]
    fn encrypt_string() {
        let crypt = CryptService::new();

        let encoded = crypt.encrypt_string(DATA).unwrap();
        assert_eq!(ENCODED_DATA.to_string(), encoded);
    }

    #[bench]
    fn bench_encrypt_string(b: &mut Bencher) {
        let crypt = CryptService::new();

        b.iter(|| {
            let _ = crypt.encrypt_string(DATA).unwrap();
        })
    }

    #[test]
    fn decrypt_string() {
        let crypt = CryptService::new();

        let encoded = crypt.encrypt_string(DATA).unwrap();
        assert_eq!(ENCODED_DATA.to_string(), encoded);

        let decoded = crypt.decrypt_string(&encoded).unwrap();
        assert_eq!(DATA.to_string(), decoded);
    }

    #[bench]
    fn bench_decrypt_string(b: &mut Bencher) {
        let crypt = CryptService::new();
        let encoded = crypt.encrypt_string(DATA).unwrap();

        b.iter(|| {
            let _ = crypt.decrypt_string(&encoded).unwrap();
        })
    }

    #[bench]
    fn bench_encrypt_and_decrypt_string(b: &mut Bencher) {
        let crypt = CryptService::new();

        b.iter(|| {
            let encoded = crypt.encrypt_string(DATA).unwrap();
            let _ = crypt.decrypt_string(&encoded).unwrap();
        })
    }

    #[test]
    fn aes_256_cbc_to_string() {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        let encrypted: Vec<u8> =
            encrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), data).unwrap();

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
            &encrypted_after_base64,
        )
        .unwrap();

        assert_eq!(data, decrypted);
        assert_eq!(DATA, String::from_utf8(decrypted).unwrap().as_str());
    }

    #[bench]
    fn bench_aes_256_cbc_to_string(b: &mut Bencher) {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();

        b.iter(|| {
            let encrypted: Vec<u8> =
                encrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), data).unwrap();

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
                &encrypted_after_base64,
            )
            .unwrap();
        });
    }

    #[test]
    fn aes_256_cbc() {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        let encrypted = encrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), data).unwrap();

        let decrypted = decrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), &encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[bench]
    fn bench_aes_256_cbc(b: &mut Bencher) {
        let cipher = Cipher::aes_256_cbc();
        let data = DATA.as_bytes();
        b.iter(|| {
            let encrypted = encrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), data).unwrap();

            let _ = decrypt(cipher, AES_256_CBC_KEY, Some(AES_256_CBC_IV), &encrypted).unwrap();
        });
    }
}
