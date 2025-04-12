use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{
    Config, KeyValueService, KeyValueServiceError, LogService, NewUser, PrivateUserData,
    SessionService, SessionServiceError, User,
};
use crate::{HashService, MysqlPool};
use actix_session::Session;
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde_derive::Deserialize;
use strum_macros::{Display, EnumString};

static FORGOT_PASSWORD_CODE_KEY: &str = "forgot_password.code";

pub struct AuthService<'a> {
    config: Data<Config>,
    db_pool: Data<MysqlPool>,
    hash: Data<HashService<'a>>,
    key_value_service: Data<KeyValueService>,
    log_service: Data<LogService>,
    session_service: Data<SessionService>,
}

impl<'a> AuthService<'a> {
    pub fn new(
        config: Data<Config>,
        db_pool: Data<MysqlPool>,
        hash: Data<HashService<'a>>,
        key_value_service: Data<KeyValueService>,
        log_service: Data<LogService>,
        session_service: Data<SessionService>,
    ) -> Self {
        Self {
            config,
            db_pool,
            hash,
            key_value_service,
            log_service,
            session_service,
        }
    }

    pub fn insert_user_id_into_session(
        &self,
        session: &Session,
        user_id: u64,
    ) -> Result<(), SessionServiceError> {
        self.session_service
            .get_ref()
            .insert(
                session,
                &self.config.get_ref().auth.user_id_session_key,
                &user_id,
            )
            .map_err(|e| {
                self.log_service
                    .get_ref()
                    .error(format!("AuthService::insert_user_id_into_session - {:}", &e).as_str());
                return e;
            })?;
        Ok(())
    }

    pub fn get_user_id_from_session(
        &self,
        session: &Session,
    ) -> Result<Option<u64>, SessionServiceError> {
        self.session_service
            .get_ref()
            .get(session, &self.config.get_ref().auth.user_id_session_key)
            .map_err(|e| {
                self.log_service
                    .get_ref()
                    .error(format!("AuthService::get_user_id_from_session - {:}", &e).as_str());
                return e;
            })
    }

    pub fn remove_user_id_from_session(&self, session: &Session) {
        self.session_service
            .get_ref()
            .remove(session, &self.config.get_ref().auth.user_id_session_key);
    }

    pub fn authenticate_by_session(&self, session: &Session) -> Result<User, AuthServiceError> {
        let user_id = self.get_user_id_from_session(session).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("AuthService::authenticate_by_session - {:}", &e).as_str());
            return AuthServiceError::AuthenticateFail;
        })?;

        match user_id {
            Some(id) => {
                let mut connection = self.db_pool.get_ref().get().map_err(|e| {
                    self.log_service
                        .get_ref()
                        .error(format!("AuthService::authenticate_by_session - {:}", &e).as_str());
                    return AuthServiceError::DbConnectionFail;
                })?;

                let user = crate::schema::users::dsl::users
                    .find(id)
                    .select(User::as_select())
                    .first(&mut connection)
                    .map_err(|e| {
                        self.log_service.get_ref().error(
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
            self.log_service
                .get_ref()
                .error(format!("AuthService::authenticate_by_credentials - {:}", &e).as_str());
            return AuthServiceError::DbConnectionFail;
        })?;

        let results: Vec<PrivateUserData> = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(&data.email))
            .select(PrivateUserData::as_select())
            .limit(1)
            .load::<PrivateUserData>(&mut connection)
            .map_err(|e| {
                self.log_service.get_ref().error(
                    format!(
                        "AuthService::authenticate_by_credentials - {} - {:}",
                        data.email, e
                    )
                    .as_str(),
                );
                return AuthServiceError::AuthenticateFail;
            })?;

        let result: Option<&PrivateUserData> = results.get(0);

        // Check auth
        let id: Option<u64> = match result {
            Some(user) => match &user.password {
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
            },
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
                        self.log_service.get_ref().error(
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
            self.log_service
                .get_ref()
                .error(format!("AuthService::register_by_credentials - {:}", &e).as_str());
            AuthServiceError::DbConnectionFail
        })?;

        diesel::insert_into(crate::schema::users::table)
            .values(new_user)
            .execute(&mut connection)
            .map_err(|e: Error| match &e {
                Error::DatabaseError(kind, _) => match &kind {
                    DatabaseErrorKind::UniqueViolation => {
                        self.log_service.get_ref().info(
                            format!(
                                "AuthService::register_by_credentials - {} - {:}",
                                &data.email, e
                            )
                            .as_str(),
                        );
                        AuthServiceError::DuplicateEmail
                    }
                    _ => {
                        self.log_service.get_ref().error(
                            format!("AuthService::register_by_credentials - {:}", &e).as_str(),
                        );
                        AuthServiceError::InsertNewUserFail
                    }
                },
                _ => {
                    self.log_service
                        .get_ref()
                        .error(format!("AuthService::register_by_credentials - {:}", &e).as_str());
                    AuthServiceError::InsertNewUserFail
                }
            })?;
        Ok(())
    }

    pub fn logout_from_session(&self, session: &Session) {
        self.remove_user_id_from_session(session);
    }

    pub fn save_forgot_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<(), KeyValueServiceError> {
        let key = format!("{}:{}", FORGOT_PASSWORD_CODE_KEY, &email);

        self.key_value_service
            .get_ref()
            .set::<&str, &str, String>(&key, code)
            .map_err(|e| {
                self.log_service.get_ref().error(
                    format!(
                        "AuthService::save_forgot_password_code - {} - {:}",
                        &key, &e
                    )
                    .as_str(),
                );
                e
            })?;
        Ok(())
    }

    pub fn get_forgot_password_code(
        &self,
        email: &str,
    ) -> Result<Option<String>, KeyValueServiceError> {
        let key = format!("{}:{}", FORGOT_PASSWORD_CODE_KEY, &email);

        let value: Option<String> = self.key_value_service.get_ref().get(&key).map_err(|e| {
            self.log_service.get_ref().error(
                format!("AuthService::get_forgot_password_code - {} - {:}", &key, &e).as_str(),
            );
            e
        })?;
        Ok(value)
    }

    pub fn is_equal_forgot_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<bool, KeyValueServiceError> {
        let stored_code: Option<String> = self.get_forgot_password_code(email).map_err(|e| {
            self.log_service.get_ref().error(
                format!(
                    "AuthService::is_equal_forgot_password_code - {} - {:}",
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
            self.log_service.get_ref().error(
                format!(
                    "AuthService::update_password_by_email - {} - {:}",
                    &email, &e
                )
                .as_str(),
            );
            AuthServiceError::Fail
        })?;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("AuthService::update_password_by_email - {:}", &e).as_str());
            AuthServiceError::DbConnectionFail
        })?;

        diesel::update(dsl_users.filter(dsl_email.eq(email)))
            .set(dsl_password.eq(hashed_password))
            .execute(&mut connection)
            .map_err(|e| {
                self.log_service.get_ref().error(
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
    AuthenticateFail,
    RegisterFail,
    CredentialsInvalid,
    DbConnectionFail,
    DuplicateEmail,
    InsertNewUserFail,
    PasswordHashFail,
    Fail,
}

// #[cfg(test)]
// mod tests {
//     use diesel::debug_query;
//     use diesel::query_builder::AsQuery;
//     use super::*;
//
//     #[test]
//     fn update_password_by_email() {
//         use crate::schema::users::dsl::users as dsl_users;
//         use crate::schema::users::dsl::email as dsl_email;
//         use crate::schema::users::dsl::password as dsl_password;
//
//         let email = "test@test.test";
//         let password = "test";
//
//
//         dbg!(&str);
//     }
// }
