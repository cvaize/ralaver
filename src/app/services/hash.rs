use actix_web::web::Data;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use strum_macros::{Display, EnumString};
use crate::LogService;

#[derive(Debug)]
pub struct HashService<'a> {
    argon2: Argon2<'a>,
    log_service: Data<LogService>
}

impl<'a> HashService<'a> {
    pub fn new(argon2: Argon2<'a>, log_service: Data<LogService>) -> Self {
        Self { argon2, log_service }
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
                self.log_service.get_ref().error(
                    format!("HashService::hash_password - {} - {:}", password, &e).as_str(),
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
