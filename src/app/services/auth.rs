use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{
    HashService, KeyValueService, KeyValueServiceError, TranslatableError, TranslatorService, User,
    UserMysqlRepository, UserService, UserServiceError,
};
use actix_web::web::Data;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub const RESET_PASSWORD_TTL: u64 = 60;
const RESET_PASSWORD_CODE_KEY: &'static str = "reset_password.code";

pub struct AuthService {
    key_value_service: Data<KeyValueService>,
    hash_service: Data<HashService>,
    user_service: Data<UserService>,
}

impl AuthService {
    pub fn new(
        key_value_service: Data<KeyValueService>,
        hash_service: Data<HashService>,
        user_service: Data<UserService>,
    ) -> Self {
        Self {
            key_value_service,
            hash_service,
            user_service,
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn login_by_password(&self, email: &str, password: &str) -> Result<u64, AuthServiceError> {
        let hash_service = self.hash_service.get_ref();
        let user_service = self.user_service.get_ref();
        let user = user_service
            .first_credentials_by_email(email)
            .map_err(|e| {
                log::error!("AuthService::login_by_password - {email} - {e}");
                return AuthServiceError::Fail;
            })?;

        if user.is_none() {
            return Err(AuthServiceError::Fail);
        }

        let user = user.unwrap();
        if user.password.is_none() {
            return Err(AuthServiceError::Fail);
        }

        let user_password_hash = user.password.unwrap();
        let is_verified = hash_service
            .verify_password(password, &user_password_hash)
            .map_err(|e| {
                log::error!("AuthService::login_by_password - {e}");
                return AuthServiceError::Fail;
            })?;

        if is_verified {
            Ok(user.id)
        } else {
            Err(AuthServiceError::Fail)
        }
    }

    pub fn register_by_credentials(&self, data: &Credentials) -> Result<(), AuthServiceError> {
        if data.is_valid() == false {
            return Err(AuthServiceError::CredentialsInvalid);
        }
        let user_service = self.user_service.get_ref();

        let user = User::empty(data.email.to_owned());

        user_service.create(user).map_err(|e| {
            log::error!("AuthService::register_by_credentials - {e}");
            match e {
                UserServiceError::DbConnectionFail => AuthServiceError::DbConnectionFail,
                UserServiceError::DuplicateEmail => AuthServiceError::DuplicateEmail,
                _ => AuthServiceError::InsertNewUserFail,
            }
        })?;

        user_service
            .update_password_by_email(&data.email, &data.password)
            .map_err(|e| {
                log::error!("AuthService::register_by_credentials - {e}");
                match e {
                    UserServiceError::DbConnectionFail => AuthServiceError::DbConnectionFail,
                    UserServiceError::PasswordHashFail => AuthServiceError::PasswordHashFail,
                    _ => AuthServiceError::InsertNewUserFail,
                }
            })?;

        Ok(())
    }

    pub fn make_reset_password_store_key(
        &self,
        email: &str,
        code: &str,
    ) -> Result<String, KeyValueServiceError> {
        let hash_service = self.hash_service.get_ref();
        let email = hash_service.hash(email);

        Ok(format!("{}.{}.{}", RESET_PASSWORD_CODE_KEY, email, code))
    }

    pub fn save_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<(), KeyValueServiceError> {
        let key = self.make_reset_password_store_key(email, code)?;
        let key_value_service = self.key_value_service.get_ref();

        key_value_service
            .set_ex(&key, true, RESET_PASSWORD_TTL)
            .map_err(|e| {
                log::error!("AuthService::save_reset_password_code - {key} - {e}");
                e
            })?;
        Ok(())
    }

    pub fn delete_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<(), KeyValueServiceError> {
        let key = self.make_reset_password_store_key(email, code)?;
        let key_value_service = self.key_value_service.get_ref();

        key_value_service.del(&key).map_err(|e| {
            log::error!("AuthService::save_reset_password_code - {key} - {e}");
            e
        })?;
        Ok(())
    }

    pub fn is_exists_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<bool, KeyValueServiceError> {
        let key = self.make_reset_password_store_key(email, code)?;
        let key_value_service = self.key_value_service.get_ref();

        let is_stored: Option<bool> = key_value_service.get(&key).map_err(|e| {
            log::error!("AuthService::is_exists_reset_password_code - {key} - {e}");
            e
        })?;

        Ok(is_stored.unwrap_or(false))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl Credentials {
    pub fn is_valid(&self) -> bool {
        Email::apply(&self.email) && MinMaxLengthString::apply(&self.password, 4, 255)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum AuthServiceError {
    CredentialsInvalid,
    DbConnectionFail,
    DuplicateEmail,
    InsertNewUserFail,
    PasswordHashFail,
    Fail,
}

impl TranslatableError for AuthServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::CredentialsInvalid => {
                translator_service.translate(lang, "error.AuthServiceError.CredentialsInvalid")
            }
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.AuthServiceError.DbConnectionFail")
            }
            Self::DuplicateEmail => {
                translator_service.translate(lang, "error.AuthServiceError.DuplicateEmail")
            }
            Self::InsertNewUserFail => {
                translator_service.translate(lang, "error.AuthServiceError.InsertNewUserFail")
            }
            Self::PasswordHashFail => {
                translator_service.translate(lang, "error.AuthServiceError.PasswordHashFail")
            }
            _ => translator_service.translate(lang, "error.AuthServiceError.Fail"),
        }
    }
}