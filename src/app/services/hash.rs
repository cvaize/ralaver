use crate::{ARGON2_SECRET_KEY};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use base64_stream::{FromBase64Reader, ToBase64Reader};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use strum_macros::{Display, EnumString};

#[derive(Debug)]
pub struct HashService<'a> {
    argon2: Argon2<'a>,
}

impl<'a> HashService<'a> {
    pub fn new() -> Self {
        let argon2 = Argon2::new_with_secret(
            ARGON2_SECRET_KEY,
            Algorithm::default(),
            Version::default(),
            Params::default(),
        )
        .unwrap();
        Self { argon2 }
    }

    pub fn verify_password(&self, password: &String, hash: &String) -> bool {
        if let Ok(hash) = PasswordHash::new(hash) {
            return self
                .argon2
                .verify_password(password.as_bytes(), &hash)
                .is_ok();
        }
        false
    }

    pub fn hash_password(&self, password: &str) -> Result<String, HashServiceError> {
        let salt = SaltString::generate(&mut OsRng);

        Ok(self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                log::error!("HashService::hash_password - {password} - {e}");
                HashServiceError::HashPasswordFail
            })?
            .to_string())
    }

    pub fn hash<T: AsRef<[u8]>>(&self, value: T) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize();
        result.to_vec()
    }

    pub fn hex_hash<T: AsRef<[u8]>>(&self, value: T) -> String {
        let value = self.hash(value);
        hex::encode(value)
    }

    pub fn base64_hash<T: AsRef<[u8]>>(&self, value: T) -> Result<String, HashServiceError> {
        let value = self.hash(value);
        self.to_base64(value)
    }

    pub fn to_base64<T: AsRef<[u8]>>(&self, value: T) -> Result<String, HashServiceError> {
        let mut reader = ToBase64Reader::new(Cursor::new(value));

        let mut base64 = String::new();
        reader.read_to_string(&mut base64).map_err(|e| {
            log::error!("HashService::to_base64 - {e}");
            HashServiceError::Fail
        })?;
        Ok(base64)
    }

    pub fn base64_to_end(&self, base64: &str) -> Result<Vec<u8>, HashServiceError> {
        let mut reader = FromBase64Reader::new(Cursor::new(base64));

        let mut result: Vec<u8> = Vec::new();

        reader.read_to_end(&mut result).map_err(|e| {
            log::error!("HashService::base64_to_end - {e}");
            HashServiceError::Fail
        })?;

        Ok(result)
    }

    pub fn base64_to_string(&self, base64: &str) -> Result<String, HashServiceError> {
        let mut reader = FromBase64Reader::new(Cursor::new(base64));

        let mut result: String = "".to_string();

        reader.read_to_string(&mut result).map_err(|e| {
            log::error!("HashService::base64_to_string - {e}");
            HashServiceError::Fail
        })?;

        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum HashServiceError {
    HashPasswordFail,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preparation;
    use base64_stream::ToBase64Reader;
    use sha2::Sha256;
    use std::io::{Cursor, Read};
    use test::Bencher;

    #[test]
    fn hash() {
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let password1 = "password123".to_string();
        let password2 = "password123".to_string();
        let password_hash1 = hash.hash(&password1);
        let password_hash2 = hash.hash(&password2);
        assert_eq!(password_hash1, password_hash2);
    }

    #[bench]
    fn bench_hash(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let value = "password123".to_string();
        b.iter(|| hash.hash(&value));
    }

    #[bench]
    fn bench_hex_hash(b: &mut Bencher) {
        // 246.71 ns/iter (+/- 3.59)
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let value = "password123".to_string();
        b.iter(|| hash.hex_hash(&value));
    }

    #[bench]
    fn bench_base64_hash(b: &mut Bencher) {
        // 257.92 ns/iter (+/- 9.32)
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let value = "password123".to_string();
        b.iter(|| hash.base64_hash(&value));
    }

    #[test]
    fn hash_to_string_by_hex() {
        let value = "password123".to_string();
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize().to_vec();
        let string = hex::encode(result);
        assert_eq!("ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f", string.as_str());
    }

    #[bench]
    fn bench_hash_to_string_by_hex(b: &mut Bencher) {
        // 155.95 ns/iter (+/- 5.62)
        let value = "password123".to_string();
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize().to_vec();
        b.iter(|| {
            let _ = hex::encode(&result);
        });
    }

    #[test]
    fn hash_to_string_by_base64() {
        let value = "password123".to_string();
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize().to_vec();

        let mut reader = ToBase64Reader::new(Cursor::new(result));

        let mut base64 = String::new();
        reader.read_to_string(&mut base64).unwrap();

        assert_eq!("75K3eLr+dx6JJFuJ7LwIpEpOFmwGZZkRiB84PURz6U8=", base64.as_str());
    }

    #[bench]
    fn bench_hash_to_string_by_base64(b: &mut Bencher) {
        // 170.26 ns/iter (+/- 5.70)
        let value = "password123".to_string();
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize().to_vec();
        b.iter(|| {
            let mut reader = ToBase64Reader::new(Cursor::new(&result));
            let mut base64 = String::new();
            reader.read_to_string(&mut base64).unwrap();
        });
    }

    #[test]
    fn verify_password() {
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let password = "password123".to_string();
        let password2 = "password".to_string();
        let password_hash = hash.hash_password(&password).unwrap();

        assert!(hash.verify_password(&password, &password_hash));
        assert!(!hash.verify_password(&password2, &password_hash));
    }

    #[bench]
    fn bench_verify_password(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let password = "password123".to_string();
        let password_hash = hash.hash_password(&password).unwrap();
        b.iter(|| hash.verify_password(&password, &password_hash));
    }

    #[bench]
    fn bench_hash_password(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let hash = all_services.hash.get_ref();

        let password = "password123".to_string();
        b.iter(|| hash.hash_password(&password).unwrap());
    }
}
