use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};
use strum_macros::{Display, EnumString};

#[derive(Debug)]
pub struct HashService {
    // TODO: Remove 'static
    argon2: Argon2<'static>,
}

impl HashService {
    pub fn new() -> Self {
        // TODO: Добавить конфиг сюда и взять из него APP_KEY в качестве ключа для argon2
        Self {
            argon2: Argon2::default(),
        }
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

    pub fn hash<T: AsRef<[u8]>>(&self, value: T) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize();
        result.to_vec()
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum HashServiceError {
    HashPasswordFail,
}

#[cfg(test)]
mod tests {
    use crate::{preparation};
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
