use crate::Config;
use actix_web::web::Data;
use base64_stream::{FromBase64Reader, ToBase64Reader};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use strum_macros::{Display, EnumString};

#[derive(Debug)]
pub struct HashService {
    argon2id_salt: [u8; 16],
}

impl HashService {
    pub fn new(config: Config) -> Self {
        Self {
            argon2id_salt: Self::gen_argon2id_salt(&config),
        }
    }

    fn gen_argon2id_salt(config: &Config) -> [u8; 16] {
        let mut app_key = config.app.key.to_owned().into_bytes();
        app_key.resize(sodoken::argon2::ARGON2_ID_SALTBYTES, 0);

        let mut salt = [0; sodoken::argon2::ARGON2_ID_SALTBYTES];
        sodoken::random::randombytes_buf(&mut salt).unwrap();
        for i in 0..sodoken::argon2::ARGON2_ID_SALTBYTES {
            let v = app_key.get(i).unwrap();
            salt[i] = v.to_owned();
        }
        salt
    }

    pub fn hash_vec<T: AsRef<[u8]>>(&self, value: T) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize();
        result.to_vec()
    }

    pub fn hash<T: AsRef<[u8]>>(&self, value: T) -> String {
        let value = self.hash_vec(value);
        hex::encode(value)
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

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, HashServiceError> {
        let new_hash = self.hash_password(password)?;
        Ok(new_hash.eq(hash))
    }

    pub fn hash_password(&self, password: &str) -> Result<String, HashServiceError> {
        let mut hash = <sodoken::SizedLockedArray<64>>::new().unwrap();

        sodoken::argon2::blocking_argon2id(
            &mut *hash.lock(),
            password.as_bytes(),
            &self.argon2id_salt,
            sodoken::argon2::ARGON2_ID_OPSLIMIT_INTERACTIVE,
            sodoken::argon2::ARGON2_ID_MEMLIMIT_INTERACTIVE,
        )
        .map_err(|e| {
            log::error!("HashService::hash_password - {e}");
            HashServiceError::Fail
        })?;

        let h = *hash.lock();
        Ok(hex::encode(h))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum HashServiceError {
    HashPasswordFail,
    Fail,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     // use crate::preparation;
//     use base64_stream::ToBase64Reader;
//     use sha2::Sha256;
//     use std::io::{Cursor, Read};
//     use test::Bencher;
//
//     // #[test]
//     // fn hash_password() {
//     //     let (_, all_services) = preparation();
//     //     let hash = all_services.hash_service.get_ref();
//     //
//     //     let password1 = "password123".to_string();
//     //     let password2 = "password123".to_string();
//     //     let password_hash1 = hash.hash_password(&password1).unwrap();
//     //     let password_hash2 = hash.hash_password(&password2).unwrap();
//     //     assert_eq!(password_hash1, password_hash2);
//     // }
//     //
//     // #[bench]
//     // fn bench_hash_password(b: &mut Bencher) {
//     //     // 67,343,054.50 ns/iter (+/- 19,188,343.94)
//     //     let (_, all_services) = preparation();
//     //     let hash = all_services.hash_service.get_ref();
//     //
//     //     let value = "password123".to_string();
//     //     b.iter(|| hash.hash_password(&value));
//     // }
//     //
//     // #[test]
//     // fn verify_password() {
//     //     let (_, all_services) = preparation();
//     //     let hash = all_services.hash_service.get_ref();
//     //
//     //     let password = "password123".to_string();
//     //     let password2 = "password".to_string();
//     //     let password_hash = hash.hash_password(&password).unwrap();
//     //
//     //     assert!(hash.verify_password(&password, &password_hash).unwrap());
//     //     assert!(!hash.verify_password(&password2, &password_hash).unwrap());
//     // }
//     //
//     // #[bench]
//     // fn bench_verify_password(b: &mut Bencher) {
//     //     // 65,405,926.30 ns/iter (+/- 3,679,559.61)
//     //     let (_, all_services) = preparation();
//     //     let hash = all_services.hash_service.get_ref();
//     //
//     //     let password = "password123".to_string();
//     //     let password_hash = hash.hash_password(&password).unwrap();
//     //     b.iter(|| hash.verify_password(&password, &password_hash));
//     // }
//     //
//     // #[bench]
//     // fn bench_hex_hash(b: &mut Bencher) {
//     //     // 246.71 ns/iter (+/- 3.59)
//     //     let (_, all_services) = preparation();
//     //     let hash = all_services.hash_service.get_ref();
//     //
//     //     let value = "password123".to_string();
//     //     b.iter(|| hash.hash(&value));
//     // }
//
//     #[test]
//     fn hash_to_string_by_hex() {
//         let value = "password123".to_string();
//         let mut hasher = Sha256::new();
//         hasher.update(value);
//         let result = hasher.finalize().to_vec();
//         let string = hex::encode(result);
//         assert_eq!(
//             "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f",
//             string.as_str()
//         );
//     }
//
//     #[bench]
//     fn bench_hash_to_string_by_hex(b: &mut Bencher) {
//         // 155.95 ns/iter (+/- 5.62)
//         let value = "password123".to_string();
//         let mut hasher = Sha256::new();
//         hasher.update(value);
//         let result = hasher.finalize().to_vec();
//         b.iter(|| {
//             let _ = hex::encode(&result);
//         });
//     }
//
//     #[test]
//     fn hash_to_string_by_base64() {
//         let value = "password123".to_string();
//         let mut hasher = Sha256::new();
//         hasher.update(value);
//         let result = hasher.finalize().to_vec();
//
//         let mut reader = ToBase64Reader::new(Cursor::new(result));
//
//         let mut base64 = String::new();
//         reader.read_to_string(&mut base64).unwrap();
//
//         assert_eq!(
//             "75K3eLr+dx6JJFuJ7LwIpEpOFmwGZZkRiB84PURz6U8=",
//             base64.as_str()
//         );
//     }
//
//     #[bench]
//     fn bench_hash_to_string_by_base64(b: &mut Bencher) {
//         // 170.26 ns/iter (+/- 5.70)
//         let value = "password123".to_string();
//         let mut hasher = Sha256::new();
//         hasher.update(value);
//         let result = hasher.finalize().to_vec();
//         b.iter(|| {
//             let mut reader = ToBase64Reader::new(Cursor::new(&result));
//             let mut base64 = String::new();
//             reader.read_to_string(&mut base64).unwrap();
//         });
//     }
// }
