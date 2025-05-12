use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{HashService, MysqlPool, UserService, UserServiceError};
use crate::{KeyValueService, KeyValueServiceError, NewUser, PrivateUserData};
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

static RESET_PASSWORD_CODE_KEY: &str = "reset_password.code";

pub struct AuthService {
    db_pool: Data<MysqlPool>,
    key_value_service: Data<KeyValueService>,
    hash_service: Data<HashService>,
    user_service: Data<UserService>,
}

impl AuthService {
    pub fn new(
        db_pool: Data<MysqlPool>,
        key_value_service: Data<KeyValueService>,
        hash_service: Data<HashService>,
        user_service: Data<UserService>,
    ) -> Self {
        Self {
            db_pool,
            key_value_service,
            hash_service,
            user_service,
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn login_by_password(
        &self,
        email: &String,
        password: &String,
    ) -> Result<u64, AuthServiceError> {
        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("AuthService::login_by_password - {e}");
            return AuthServiceError::DbConnectionFail;
        })?;

        let user: PrivateUserData = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .map_err(|e| {
                if e.to_string() != "Record not found" {
                    log::error!("AuthService::login_by_password - {email} - {e}");
                }
                return AuthServiceError::Fail;
            })?;

        if user.password.is_none() {
            return Err(AuthServiceError::Fail);
        }
        let user_password_hash = user.password.unwrap();
        let is_verified = self
            .hash_service
            .get_ref()
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

        let mut new_user = NewUser::empty(data.email.to_owned());
        new_user.password = Some(data.password.to_owned());

        user_service.insert(new_user).map_err(|e| {
            log::error!("AuthService::register_by_credentials - {e}");
            match e {
                UserServiceError::DbConnectionFail => AuthServiceError::DbConnectionFail,
                UserServiceError::PasswordHashFail => AuthServiceError::PasswordHashFail,
                UserServiceError::DuplicateEmail => AuthServiceError::DuplicateEmail,
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
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::password as dsl_password;
        use crate::schema::users::dsl::users as dsl_users;

        let hashed_password = self
            .hash_service
            .get_ref()
            .hash_password(password)
            .map_err(|e| {
                log::error!("AuthService::update_password_by_email - {email} - {e}",);
                AuthServiceError::Fail
            })?;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("AuthService::update_password_by_email - {e}",);
            AuthServiceError::DbConnectionFail
        })?;

        diesel::update(dsl_users.filter(dsl_email.eq(email)))
            .set(dsl_password.eq(hashed_password))
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("AuthService::update_password_by_email - {email} - {e}");
                AuthServiceError::Fail
            })?;
        Ok(())
    }

    pub fn exists_user_by_email(&self, email: &str) -> Result<bool, AuthServiceError> {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        use diesel::dsl::exists;
        use diesel::select;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("AuthService::exists_user_by_email - {e}");
            AuthServiceError::DbConnectionFail
        })?;

        let email_exists: bool = select(exists(dsl_users.filter(dsl_email.eq(email))))
            .get_result(&mut connection)
            .map_err(|e| {
                log::error!("AuthService::exists_user_by_email - {e}");
                AuthServiceError::Fail
            })?;

        Ok(email_exists)
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

#[cfg(test)]
mod tests {
    use crate::{preparation, Credentials, PrivateUserData};
    #[allow(unused_imports)]
    use diesel::prelude::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

    #[test]
    fn exists_user_by_email() {
        let (_, all_services) = preparation();

        assert_eq!(
            false,
            all_services
                .auth
                .exists_user_by_email("null@null.null")
                .unwrap()
        );
        assert_eq!(
            true,
            all_services
                .auth
                .exists_user_by_email("admin@admin.example")
                .unwrap()
        );
    }

    #[test]
    fn update_password_by_email() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;

        let (all_connections, all_services) = preparation();

        let email = "admin@admin.example";

        let password = all_services.rand.get_ref().str(64);
        all_services
            .auth
            .update_password_by_email(email, &password)
            .unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        let user_password_hash = user.password.clone().unwrap();

        assert_eq!(
            true,
            all_services
                .hash
                .get_ref()
                .verify_password(&password, &user_password_hash)
                .unwrap()
        );

        let password = all_services.rand.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .hash
                .get_ref()
                .verify_password(&password, &user_password_hash)
                .unwrap()
        );
    }

    #[test]
    fn reset_password_code() {
        let (_, all_services) = preparation();

        let email = "admin@admin.example";
        let code = all_services.rand.get_ref().str(64);
        all_services
            .auth
            .save_reset_password_code(email, &code)
            .unwrap();

        let saved_code = all_services
            .auth
            .get_reset_password_code(email)
            .unwrap()
            .unwrap();
        assert_eq!(code, saved_code);
        assert_eq!(
            true,
            all_services
                .auth
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
        let code = all_services.rand.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .auth
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
    }

    #[test]
    fn register_by_credentials() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        let (all_connections, all_services) = preparation();

        let password = all_services.rand.get_ref().str(64);
        let email = format!("admin{}@admin.example", &password);

        let cred = Credentials {
            email: email.to_owned(),
            password: password.to_owned(),
        };
        all_services.auth.register_by_credentials(&cred).unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email.to_owned()))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        assert_eq!(user.email, email);
    }
}
