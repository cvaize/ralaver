use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{HashService, KeyValueService, KeyValueServiceError, TranslatableError, TranslatorService, User, UserMysqlRepository, UserService, UserServiceError};
use actix_web::web::Data;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

const RESET_PASSWORD_CODE_KEY: &'static str = "reset_password.code";

pub struct AuthService {
    key_value_service: Data<KeyValueService>,
    hash_service: Data<HashService>,
    user_service: Data<UserService>,
    user_repository: Data<UserMysqlRepository>,
}

impl AuthService {
    pub fn new(
        key_value_service: Data<KeyValueService>,
        hash_service: Data<HashService>,
        user_service: Data<UserService>,
        user_repository: Data<UserMysqlRepository>,
    ) -> Self {
        Self {
            key_value_service,
            hash_service,
            user_service,
            user_repository,
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn login_by_password(&self, email: &str, password: &str) -> Result<u64, AuthServiceError> {
        let hash_service = self.hash_service.get_ref();
        let user_repository = self.user_repository.get_ref();
        let user = user_repository.first_credentials_by_email(email).map_err(|e| {
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

        user_service.create(&user).map_err(|e| {
            log::error!("AuthService::register_by_credentials - {e}");
            match e {
                UserServiceError::DbConnectionFail => AuthServiceError::DbConnectionFail,
                UserServiceError::DuplicateEmail => AuthServiceError::DuplicateEmail,
                _ => AuthServiceError::InsertNewUserFail,
            }
        })?;

        user_service.update_password_by_email(&data.email, &data.password).map_err(|e| {
            log::error!("AuthService::register_by_credentials - {e}");
            match e {
                UserServiceError::DbConnectionFail => AuthServiceError::DbConnectionFail,
                UserServiceError::PasswordHashFail => AuthServiceError::PasswordHashFail,
                _ => AuthServiceError::InsertNewUserFail,
            }
        })?;

        Ok(())
    }

    pub fn save_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<(), KeyValueServiceError> {
        let key = format!("{}:{}", RESET_PASSWORD_CODE_KEY, &email);

        self.key_value_service
            .get_ref()
            .set(&key, code)
            .map_err(|e| {
                log::error!("AuthService::save_reset_password_code - {key} - {e}");
                e
            })?;
        Ok(())
    }

    pub fn get_reset_password_code(
        &self,
        email: &str,
    ) -> Result<Option<String>, KeyValueServiceError> {
        let key = format!("{}:{}", RESET_PASSWORD_CODE_KEY, &email);

        let value: Option<String> = self.key_value_service.get_ref().get(&key).map_err(|e| {
            log::error!("AuthService::get_reset_password_code - {key} - {e}");
            e
        })?;
        Ok(value)
    }

    pub fn is_equal_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<bool, KeyValueServiceError> {
        let stored_code: Option<String> = self.get_reset_password_code(email).map_err(|e| {
            log::error!("AuthService::is_equal_reset_password_code - {email} - {e}");
            e
        })?;
        match stored_code {
            Some(stored_code) => Ok(stored_code.eq(code)),
            _ => Ok(false),
        }
    }

    pub fn update_password_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), AuthServiceError> {
        let user_repository = self.user_repository.get_ref();
        let hash_service = self.hash_service.get_ref();

        let hashed_password = hash_service.hash_password(password).map_err(|e| {
            log::error!("AuthService::update_password_by_email - {email} - {e}",);
            AuthServiceError::Fail
        })?;

        user_repository
            .update_password_by_email(email, &hashed_password)
            .map_err(|e| {
                log::error!("AuthService::update_password_by_email - {email} - {e}",);
                AuthServiceError::Fail
            })?;

        Ok(())
    }

    pub fn exists_user_by_email(&self, email: &str) -> Result<bool, AuthServiceError> {
        self.user_repository
            .get_ref()
            .exists_by_email(email)
            .map_err(|_| AuthServiceError::Fail)
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
    fn translate(&self, lang: &str, translate_service: &TranslatorService) -> String {
        match self {
            Self::CredentialsInvalid => {
                translate_service.translate(lang, "error.AuthServiceError.CredentialsInvalid")
            }
            Self::DbConnectionFail => {
                translate_service.translate(lang, "error.AuthServiceError.DbConnectionFail")
            }
            Self::DuplicateEmail => {
                translate_service.translate(lang, "error.AuthServiceError.DuplicateEmail")
            }
            Self::InsertNewUserFail => {
                translate_service.translate(lang, "error.AuthServiceError.InsertNewUserFail")
            }
            Self::PasswordHashFail => {
                translate_service.translate(lang, "error.AuthServiceError.PasswordHashFail")
            }
            _ => translate_service.translate(lang, "error.AuthServiceError.Fail"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::preparation;

    #[test]
    fn exists_user_by_email() {
        let (_, all_services) = preparation();

        assert_eq!(
            false,
            all_services
                .auth_service
                .exists_user_by_email("null@null.null")
                .unwrap()
        );
        assert_eq!(
            true,
            all_services
                .auth_service
                .exists_user_by_email("admin@admin.example")
                .unwrap()
        );
    }

    #[test]
    fn reset_password_code() {
        let (_, all_services) = preparation();

        let email = "admin@admin.example";
        let code = all_services.rand_service.get_ref().str(64);
        all_services
            .auth_service
            .save_reset_password_code(email, &code)
            .unwrap();

        let saved_code = all_services
            .auth_service
            .get_reset_password_code(email)
            .unwrap()
            .unwrap();
        assert_eq!(code, saved_code);
        assert_eq!(
            true,
            all_services
                .auth_service
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
        let code = all_services.rand_service.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .auth_service
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
    }
}
