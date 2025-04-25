use crate::helpers::vec_into_array;
use crate::{log_map_err, Config, RandomService};
use actix_web::web::Data;
use base64_stream::FromBase64Reader;
use base64_stream::ToBase64Reader;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::io::{Cursor, Read};
use strum_macros::{Display, EnumString};

pub struct CryptService {
    cipher: Cipher,
    cipher_key: [u8; 32],
}

impl CryptService {
    pub fn new(config: Data<Config>) -> Self {
        if config.get_ref().app.key.len() == 0 {
            panic!("APP_KEY is missing!");
        }
        let cipher_key = Self::parse_key(&config.get_ref().app.key);
        Self {
            cipher: Cipher::aes_256_cbc(),
            cipher_key,
        }
    }

    pub fn random_key() -> [u8; 32] {
        let rand = RandomService::new();
        rand.bytes_32()
    }

    pub fn random_key_string() -> String {
        let key = Self::random_key();

        let mut reader = ToBase64Reader::new(Cursor::new(key));
        let mut base64 = String::new();
        reader.read_to_string(&mut base64).unwrap();

        base64
    }

    fn parse_key(key: &str) -> [u8; 32] {
        let mut reader = FromBase64Reader::new(Cursor::new(key));

        let mut key: Vec<u8> = vec![];
        reader.read_to_end(&mut key).unwrap();

        vec_into_array(key)
    }

    pub fn encrypt_string(&self, string: &str, password: Option<&str>) -> Result<String, CryptServiceError> {
        let password_ = if let Some(p) = password { Some(p.as_bytes()) } else { None };
        let encrypted: Vec<u8> = encrypt(
            self.cipher,
            &self.cipher_key,
            password_,
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

    pub fn decrypt_string(&self, string: &str, password: Option<&str>) -> Result<String, CryptServiceError> {
        let password_ = if let Some(p) = password { Some(p.as_bytes()) } else { None };
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
            &self.cipher_key,
            password_,
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
    use crate::{preparation, CryptService};
    use base64_stream::FromBase64Reader;
    use base64_stream::ToBase64Reader;
    use openssl::symm::{decrypt, encrypt, Cipher};
    use std::io::{Cursor, Read};
    use test::Bencher;

    static DATA: &str =
        "1-10459396685910126978-DLum2QqN6bjg8L7kfMrORdazvv4dlrOT0Z9XcEDZMJ5DAnISYZx18wTHvNI5mlH2";
    static ENCODED_DATA: &str = "ipCyMRF/ya6s++CijbRF8gRYtZ7ZWFOgZ87xwDRpr3XAJJpBr5H0ZN/WWH1TPuuFzwKpfONsc0SSy1Vk/AZKiFHizodDtc112M0tQgcYxAGK4FayiRl5fnZBQzt8Cssk";
    static ENCODED_PASSWORD_DATA: &str = "/TTqk91ZFpl7zvxNEoD81MRTTizGCeeTXyol9ZijAVNKZwY2QjQ3XLptBy9H/2LFzNoVcjmTqfzW2JyMrdQZONig3QjohwEW2kyVXWFkhoTqLNC6Mq2kTjQftSV3oH0x";
    static AES_256_CBC_KEY: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    static AES_256_CBC_IV: &[u8; 64] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    static PASSWORD: &str = "passwordpasswordpassword";
    static PASSWORD2: &str = "pawordpassworsswordpassd";
    // TODO: https://security.stackexchange.com/questions/35210/encrypting-using-aes-256-do-i-need-iv
    // Если вы используете каждый ключ только один раз, то не используйте IV. Если вы используете ключ несколько раз, вам следует использовать каждый раз другой IV, чтобы пара (ключ, IV) не использовалась повторно.
    //
    // Точные требования к IV зависят от выбранного режима цепочки, но обычно достаточно случайного 128-битного значения. Оно должно быть разным для каждого сообщения, которое вы шифруете. Сохраните его вместе с зашифрованным текстом, обычно в качестве префикса.

    #[test]
    fn parse_key() {
        let mut reader = ToBase64Reader::new(Cursor::new(AES_256_CBC_KEY));

        let mut base64 = String::new();
        reader.read_to_string(&mut base64).unwrap();

        assert_eq!(&CryptService::parse_key(&base64), AES_256_CBC_KEY);
    }

    #[test]
    fn random_key() {
        assert_eq!(CryptService::random_key().len(), 32);
    }

    #[test]
    fn random_key_string() {
        assert!(CryptService::random_key_string().len() > 32);
    }

    #[test]
    fn encrypt_string() {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        let encoded = crypt.encrypt_string(DATA, None).unwrap();
        assert_eq!(ENCODED_DATA.to_string(), encoded);
        let encoded = crypt.encrypt_string(DATA, Some(PASSWORD)).unwrap();
        assert_eq!(ENCODED_PASSWORD_DATA.to_string(), encoded);
    }

    #[bench]
    fn bench_encrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        b.iter(|| {
            let _ = crypt.encrypt_string(DATA, Some(PASSWORD)).unwrap();
        })
    }

    #[test]
    fn decrypt_string() {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        let encoded = crypt.encrypt_string(DATA, Some(PASSWORD)).unwrap();
        assert_eq!(ENCODED_PASSWORD_DATA.to_string(), encoded);

        let decoded = crypt.decrypt_string(&encoded, Some(PASSWORD)).unwrap();
        assert_eq!(DATA.to_string(), decoded);
        assert_ne!(DATA.to_string(), crypt.decrypt_string(&encoded, None).unwrap());
        assert_ne!(DATA.to_string(), crypt.decrypt_string(&encoded, Some(PASSWORD2)).unwrap());
    }

    #[bench]
    fn bench_decrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();
        let encoded = crypt.encrypt_string(DATA, Some(PASSWORD)).unwrap();

        b.iter(|| {
            let _ = crypt.decrypt_string(&encoded, Some(PASSWORD)).unwrap();
        })
    }

    #[bench]
    fn bench_encrypt_and_decrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        b.iter(|| {
            let encoded = crypt.encrypt_string(DATA, Some(PASSWORD)).unwrap();
            let _ = crypt.decrypt_string(&encoded, Some(PASSWORD)).unwrap();
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
