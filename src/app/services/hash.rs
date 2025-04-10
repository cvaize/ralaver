use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

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
            return self.argon2.verify_password(password.as_bytes(), &hash).is_ok();
        }
        false
    }

    pub fn hash_password(&self, password: &str) -> Result<String, MakePasswordHashFail>{
        let salt = SaltString::generate(&mut OsRng);

        Ok(self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| MakePasswordHashFail)?
            .to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MakePasswordHashFail;