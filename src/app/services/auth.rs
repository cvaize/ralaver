use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{Config, KeyValueService, KeyValueServiceError, NewUser, PrivateUserData, User};
use crate::{HashService, MysqlPool};
use crate::{Session, SessionService};
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde_derive::Deserialize;
use strum_macros::{Display, EnumString};


static RESET_PASSWORD_CODE_KEY: &str = "reset_password.code";

pub struct AuthService<'a> {
    #[allow(dead_code)]
    config: Data<Config>,
    db_pool: Data<MysqlPool>,
    hash: Data<HashService<'a>>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
}

impl<'a> AuthService<'a> {
    pub fn new(
        config: Data<Config>,
        db_pool: Data<MysqlPool>,
        hash: Data<HashService<'a>>,
        session_service: Data<SessionService>,
        key_value_service: Data<KeyValueService>,
    ) -> Self {
        Self {
            config,
            db_pool,
            hash,
            session_service,
            key_value_service,
        }
    }

    pub fn save_user_id_into_session(
        &self,
        session: &mut Session,
        user_id: u64,
    ) -> Result<(), AuthServiceError> {
        let session_service = self.session_service.get_ref();

        session.user_id = user_id;
        session_service.save_session(session).map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::set_user_id_into_session - {:}", &e).as_str()
            );
            return AuthServiceError::SessionError;
        })?;

        Ok(())
    }

    pub fn authenticate_by_session(&self, session: &Session) -> Result<User, AuthServiceError> {
        let user_id: Option<u64> = if session.user_id == 0 {
            None
        } else {
            Some(session.user_id)
        };

        match user_id {
            Some(id) => {
                let mut connection = self.db_pool.get_ref().get().map_err(|e| {
                    log::error!(
                        "{}",
                        format!("AuthService::authenticate_by_session - {:}", &e).as_str()
                    );
                    return AuthServiceError::DbConnectionFail;
                })?;

                let user = crate::schema::users::dsl::users
                    .find(id)
                    .select(User::as_select())
                    .first(&mut connection)
                    .map_err(|e| {
                        log::error!(
                            "{}",
                            format!("AuthService::authenticate_by_session - {:}", &e).as_str(),
                        );
                        return AuthServiceError::AuthenticateFail;
                    })?;

                Ok(user)
            }
            _ => Err(AuthServiceError::AuthenticateFail),
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn authenticate_by_credentials(&self, data: &Credentials) -> Result<u64, AuthServiceError> {
        if data.is_valid() == false {
            return Err(AuthServiceError::AuthenticateFail);
        }

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::authenticate_by_credentials - {:}", &e).as_str()
            );
            return AuthServiceError::DbConnectionFail;
        })?;

        let user: PrivateUserData = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(&data.email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .map_err(|e| {
                if e.to_string() != "Record not found" {
                    log::error!(
                        "{}",
                        format!(
                            "AuthService::authenticate_by_credentials - {} - {:}",
                            data.email, e
                        )
                        .as_str(),
                    );
                }
                return AuthServiceError::AuthenticateFail;
            })?;

        // Check auth
        let id: Option<u64> = match &user.password {
            Some(user_password_hash) => {
                if self
                    .hash
                    .get_ref()
                    .verify_password(&data.password, user_password_hash)
                {
                    Some(user.id)
                } else {
                    None
                }
            }
            _ => None,
        };

        match id {
            Some(id) => Ok(id),
            _ => Err(AuthServiceError::AuthenticateFail),
        }
    }

    pub fn register_by_credentials(&self, data: &Credentials) -> Result<(), AuthServiceError> {
        if data.is_valid() == false {
            return Err(AuthServiceError::CredentialsInvalid);
        }
        let new_user = NewUser {
            email: data.email.to_owned(),
            password: Some(
                self.hash
                    .get_ref()
                    .hash_password(&data.password)
                    .map_err(|e| {
                        log::error!(
                            "{}",
                            format!(
                                "AuthService::register_by_credentials - {} - {:}",
                                data.password, e
                            )
                            .as_str(),
                        );
                        AuthServiceError::PasswordHashFail
                    })?,
            ),
        };

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::register_by_credentials - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        diesel::insert_into(crate::schema::users::table)
            .values(new_user)
            .execute(&mut connection)
            .map_err(|e: Error| match &e {
                Error::DatabaseError(kind, _) => match &kind {
                    DatabaseErrorKind::UniqueViolation => {
                        log::info!(
                            "{}",
                            format!(
                                "AuthService::register_by_credentials - {} - {:}",
                                &data.email, e
                            )
                            .as_str(),
                        );
                        AuthServiceError::DuplicateEmail
                    }
                    _ => {
                        log::error!(
                            "{}",
                            format!("AuthService::register_by_credentials - {:}", &e).as_str(),
                        );
                        AuthServiceError::InsertNewUserFail
                    }
                },
                _ => {
                    log::error!(
                        "{}",
                        format!("AuthService::register_by_credentials - {:}", &e).as_str()
                    );
                    AuthServiceError::InsertNewUserFail
                }
            })?;
        Ok(())
    }

    pub fn logout_from_session(&self, session: &Session) -> Result<(), AuthServiceError> {
        let session_service = self.session_service.get_ref();
        session_service.delete_session(session).map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::logout_from_session - {:}", &e).as_str()
            );
            return AuthServiceError::LogoutFail;
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
                log::error!(
                    "{}",
                    format!("AuthService::save_reset_password_code - {} - {:}", &key, &e).as_str(),
                );
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
            log::error!(
                "{}",
                format!("AuthService::get_reset_password_code - {} - {:}", &key, &e).as_str(),
            );
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
            log::error!(
                "{}",
                format!(
                    "AuthService::is_equal_reset_password_code - {} - {:}",
                    email, e
                )
                .as_str(),
            );
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

        let hashed_password = self.hash.get_ref().hash_password(password).map_err(|e| {
            log::error!(
                "{}",
                format!(
                    "AuthService::update_password_by_email - {} - {:}",
                    &email, &e
                )
                .as_str(),
            );
            AuthServiceError::Fail
        })?;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::update_password_by_email - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        diesel::update(dsl_users.filter(dsl_email.eq(email)))
            .set(dsl_password.eq(hashed_password))
            .execute(&mut connection)
            .map_err(|e| {
                log::error!(
                    "{}",
                    format!(
                        "AuthService::update_password_by_email - {} - {:}",
                        &email, &e
                    )
                    .as_str(),
                );
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
            log::error!(
                "{}",
                format!("AuthService::exists_user_by_email - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        let email_exists: bool = select(exists(dsl_users.filter(dsl_email.eq(email))))
            .get_result(&mut connection)
            .map_err(|e| {
                log::error!(
                    "{}",
                    format!("AuthService::exists_user_by_email - {:}", &e).as_str()
                );
                AuthServiceError::Fail
            })?;

        Ok(email_exists)
    }
}

#[derive(Deserialize, Debug)]
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
    SessionError,
    AuthenticateFail,
    RegisterFail,
    CredentialsInvalid,
    DbConnectionFail,
    DuplicateEmail,
    InsertNewUserFail,
    PasswordHashFail,
    LogoutFail,
    Fail,
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::{preparation, Credentials, PrivateUserData};
    #[allow(unused_imports)]
    use diesel::prelude::*;
    #[allow(unused_imports)]
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
    #[allow(unused_imports)]
    use tokio;

    // Used in manual mode
    // #[tokio::test]
    #[allow(dead_code)]
    async fn exists_user_by_email() {
        let (_, all_services) = preparation().await;

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

    // Used in manual mode
    // #[tokio::test]
    #[allow(dead_code)]
    async fn update_password_by_email() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;

        let (all_connections, all_services) = preparation().await;

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
        );

        let password = all_services.rand.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .hash
                .get_ref()
                .verify_password(&password, &user_password_hash)
        );
    }

    // Used in manual mode
    // #[tokio::test]
    #[allow(dead_code)]
    async fn reset_password_code() {
        let (_, all_services) = preparation().await;

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

    // Used in manual mode
    // #[tokio::test]
    #[allow(dead_code)]
    async fn authenticate_by_credentials() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        let (all_connections, all_services) = preparation().await;

        let email = "admin@admin.example";
        let password = all_services.rand.get_ref().str(64);

        all_services
            .auth
            .update_password_by_email(email, &password)
            .unwrap();

        let cred = Credentials {
            email: email.to_owned(),
            password: password.to_owned(),
        };
        let user_id = all_services
            .auth
            .authenticate_by_credentials(&cred)
            .unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        assert_eq!(user.id, user_id);
    }

    // Used in manual mode
    // #[tokio::test]
    #[allow(dead_code)]
    async fn register_by_credentials() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        let (all_connections, all_services) = preparation().await;

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
