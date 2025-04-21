use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use strum_macros::{Display, EnumString};

#[derive(Debug)]
pub struct HashService<'a> {
    argon2: Argon2<'a>,
}

impl<'a> HashService<'a> {
    pub fn new(argon2: Argon2<'a>) -> Self {
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
                log::error!(
                    "{}",
                    format!("HashService::hash_password - {} - {:}", password, &e).as_str()
                );
                HashServiceError::HashPasswordFail
            })?
            .to_string())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum HashServiceError {
    HashPasswordFail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn verify_password() {
        let hash = HashService::new(Argon2::default());

        let password = "password123".to_string();
        let password2 = "password".to_string();
        let password_hash = hash.hash_password(&password).unwrap();

        assert!(hash.verify_password(&password, &password_hash));
        assert!(!hash.verify_password(&password2, &password_hash));
    }

    #[bench]
    fn bench_verify_password(b: &mut Bencher) {
        let hash = HashService::new(Argon2::default());
        let password = "password123".to_string();
        let password_hash = hash.hash_password(&password).unwrap();
        b.iter(|| hash.verify_password(&password, &password_hash));
    }

    #[bench]
    fn bench_hash_password(b: &mut Bencher) {
        let hash = HashService::new(Argon2::default());
        let password = "password123".to_string();
        b.iter(|| hash.hash_password(&password).unwrap());
    }

    // #[test]
    // fn test_sodoken() {
    //     use sodoken::*;
    //
    //     let mut pub_key = [0; sign::PUBLICKEYBYTES];
    //     let mut sec_key = SizedLockedArray::new().unwrap();
    //
    //     sign::keypair(&mut pub_key, &mut sec_key.lock()).unwrap();
    //
    //     let mut sig = [0; sign::SIGNATUREBYTES];
    //
    //     sign::sign_detached(&mut sig, b"hello", &sec_key.lock()).unwrap();
    //     assert!(sign::verify_detached(&sig, b"hello", &pub_key));
    //     assert!(!sign::verify_detached(&sig, b"world", &pub_key));
    // }
    //
    // #[bench]
    // fn bench_sodoken(b: &mut Bencher) {
    //     use sodoken::*;
    //
    //     let mut pub_key = [0; sign::PUBLICKEYBYTES];
    //     let mut sec_key = SizedLockedArray::new().unwrap();
    //
    //     sign::keypair(&mut pub_key, &mut sec_key.lock()).unwrap();
    //
    //     let mut sig = [0; sign::SIGNATUREBYTES];
    //     let password = b"password123";
    //
    //     sign::sign_detached(&mut sig, password, &sec_key.lock()).unwrap();
    //
    //     b.iter(|| sign::verify_detached(&sig, password, &pub_key));
    // }
}
