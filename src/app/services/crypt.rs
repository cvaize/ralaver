use crate::helpers::vec_into_array;
use crate::{Config, HashService, RandomService};
use actix_web::web::Data;
use base64_stream::FromBase64Reader;
use base64_stream::ToBase64Reader;
use serde_derive::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedData {
    pub iv: String,
    pub value: String,
    pub mac: String,
}

pub struct CryptService<'a> {
    random_service: Data<RandomService>,
    hash_service: Data<HashService<'a>>,
    cipher: openssl::symm::Cipher,
    cipher_key_string: String,
    cipher_key: [u8; 32],
}

impl<'a> CryptService<'a> {
    pub fn new(
        config: Data<Config>,
        random_service: Data<RandomService>,
        hash_service: Data<HashService<'a>>,
    ) -> Self {
        if config.get_ref().app.key.len() == 0 {
            panic!("APP_KEY is missing!");
        }
        let cipher_key_string: String = config.get_ref().app.key.to_owned();
        let cipher_key: [u8; 32] = Self::parse_key(&cipher_key_string);
        Self {
            random_service,
            hash_service,
            cipher: openssl::symm::Cipher::aes_256_cbc(),
            cipher_key_string,
            cipher_key,
        }
    }

    pub fn random_key() -> String {
        let key: [u8; 32] = RandomService::new().bytes_32();

        let mut reader = ToBase64Reader::new(Cursor::new(key));
        let mut base64 = String::new();
        reader.read_to_string(&mut base64).unwrap();

        base64
    }

    fn parse_key(key: &str) -> [u8; 32] {
        let mut reader = FromBase64Reader::new(Cursor::new(key));

        let mut key: Vec<u8> = Vec::new();
        reader.read_to_end(&mut key).unwrap();

        vec_into_array(key)
    }

    fn hash(&self, iv: &str, value: &str, key: &str) -> Result<String, CryptServiceError> {
        let mut s = iv.to_owned();
        s.push_str(value);
        s.push_str(key);
        let hash_service = self.hash_service.get_ref();
        let hash = hash_service.hash(s);
        hash_service.to_base64(hash).map_err(|e| {
            log::error!("CryptService::hash - {e}");
            CryptServiceError::Fail
        })
    }

    pub fn encrypt_string(&self, string: &str) -> Result<String, CryptServiceError> {
        let hash_service = self.hash_service.get_ref();
        let iv: [u8; 128] = self.random_service.get_ref().bytes_128();
        let value: Vec<u8> =
            openssl::symm::encrypt(self.cipher, &self.cipher_key, Some(&iv), string.as_bytes())
                .map_err(|e| {
                    log::error!("CryptService::encrypt_string - {e}");
                    CryptServiceError::Fail
                })?;

        let iv_base64: String = hash_service.to_base64(iv).map_err(|e| {
            log::error!("CryptService::encrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let value_base64: String = hash_service.to_base64(value).map_err(|e| {
            log::error!("CryptService::encrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let mac: String = self
            .hash(&iv_base64, &value_base64, &self.cipher_key_string)
            .map_err(|e| {
                log::error!("CryptService::encrypt_string - {e}");
                CryptServiceError::Fail
            })?;

        let data = EncryptedData {
            iv: iv_base64,
            value: value_base64,
            mac,
        };

        let data_string: String = serde_json::to_string(&data).map_err(|e| {
            log::error!("CryptService::encrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let data_base64: String = hash_service.to_base64(data_string).map_err(|e| {
            log::error!("CryptService::encrypt_string - {e}");
            CryptServiceError::Fail
        })?;

        Ok(data_base64)
    }

    pub fn decrypt_string(&self, data_base64: &str) -> Result<String, CryptServiceError> {
        let hash_service = self.hash_service.get_ref();
        let data_string: String = hash_service.base64_to_string(data_base64).map_err(|e| {
            log::error!("CryptService::decrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let data: EncryptedData = serde_json::from_str(&data_string).map_err(|e| {
            log::error!("CryptService::decrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let mac: String = self
            .hash(&data.iv, &data.value, &self.cipher_key_string)
            .map_err(|e| {
                log::error!("CryptService::decrypt_string - {e}");
                CryptServiceError::Fail
            })?;

        if mac.ne(&data.mac) {
            return Err(CryptServiceError::Fail);
        }

        let iv: Vec<u8> = hash_service.base64_to_end(&data.iv).map_err(|e| {
            log::error!("CryptService::decrypt_string - {e}");
            CryptServiceError::Fail
        })?;
        let value: Vec<u8> = hash_service.base64_to_end(&data.value).map_err(|e| {
            log::error!("CryptService::decrypt_string - {e}");
            CryptServiceError::Fail
        })?;

        let decrypted = openssl::symm::decrypt(self.cipher, &self.cipher_key, Some(&iv), &value)
            .map_err(|e| {
                log::error!("CryptService::decrypt_string - {e}");
                CryptServiceError::Fail
            })?;

        String::from_utf8(decrypted).map_err(|e| {
            log::error!("CryptService::decrypt_string - {e}");
            CryptServiceError::Fail
        })
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
    static AES_256_CBC_KEY: &[u8; 32] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    static AES_256_CBC_IV: &[u8; 64] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

    #[test]
    fn parse_key() {
        let mut reader = ToBase64Reader::new(Cursor::new(AES_256_CBC_KEY));

        let mut base64 = String::new();
        reader.read_to_string(&mut base64).unwrap();

        assert_eq!(&CryptService::parse_key(&base64), AES_256_CBC_KEY);
    }

    #[test]
    fn random_key() {
        assert!(CryptService::random_key().len() > 0);
    }

    #[test]
    fn encrypt_string() {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        let encoded = crypt.encrypt_string(DATA).unwrap();
        assert!(encoded.len() > 1);
    }

    #[bench]
    fn bench_encrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        b.iter(|| {
            let _ = crypt.encrypt_string(DATA).unwrap();
        })
    }

    #[test]
    fn decrypt_string() {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

        let encoded: String = crypt.encrypt_string(DATA).unwrap();

        let decoded: String = crypt.decrypt_string(&encoded).unwrap();
        assert_eq!(DATA.to_string(), decoded);
    }

    #[bench]
    fn bench_decrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();
        let encoded = crypt.encrypt_string(DATA).unwrap();

        b.iter(|| {
            let _ = crypt.decrypt_string(&encoded).unwrap();
        })
    }

    #[bench]
    fn bench_encrypt_and_decrypt_string(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let crypt = all_services.crypt.get_ref();

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

        let mut encrypted_after_base64: Vec<u8> = Vec::new();

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

            let mut encrypted_after_base64: Vec<u8> = Vec::new();

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
