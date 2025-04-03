use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{Config, NewUser, PrivateUserData, User};
use crate::{DbPool, HashService};
use actix_session::{Session, SessionGetError, SessionInsertError};
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde_derive::Deserialize;

#[derive(Debug)]
pub struct AuthService<'a> {
    config: Data<Config>,
    db_pool: Data<DbPool>,
    hash: Data<HashService<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum AuthError {
    AuthenticateFail,
    RegisterFail,
    CredentialsInvalid,
    DbConnectionFail,
    DuplicateEmail,
    InsertNewUserFail,
    HashingPasswordFail,
}

impl<'a> AuthService<'a> {
    pub fn new(config: Data<Config>, db_pool: Data<DbPool>, hash: Data<HashService<'a>>) -> Self {
        Self {
            config,
            db_pool,
            hash,
        }
    }

    pub fn insert_user_id_into_session(
        &self,
        session: &Session,
        user_id: u64,
    ) -> Result<(), SessionInsertError> {
        session.insert(&self.config.get_ref().auth.user_id_session_key, user_id)
    }

    pub fn get_user_id_from_session(
        &self,
        session: &Session,
    ) -> Result<Option<u64>, SessionGetError> {
        session.get::<u64>(&self.config.get_ref().auth.user_id_session_key)
    }

    pub fn remove_user_id_from_session(&self, session: &Session) {
        session.remove(&self.config.get_ref().auth.user_id_session_key);
    }

    pub fn authenticate_by_session(&self, session: &Session) -> Result<User, AuthError> {
        let user_id = self
            .get_user_id_from_session(session)
            .map_err(|_| AuthError::AuthenticateFail)?;

        match user_id {
            Some(id) => {
                let mut connection = self
                    .db_pool
                    .get_ref()
                    .get()
                    .map_err(|_| AuthError::AuthenticateFail)?;

                let user = crate::schema::users::dsl::users
                    .find(id)
                    .select(User::as_select())
                    .first(&mut connection)
                    .map_err(|_| AuthError::AuthenticateFail)?;

                Ok(user)
            }
            _ => Err(AuthError::AuthenticateFail),
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn authenticate_by_credentials(&self, data: &Credentials) -> Result<u64, AuthError> {
        if data.is_valid() == false {
            return Err(AuthError::AuthenticateFail);
        }

        let mut connection = self
            .db_pool
            .get_ref()
            .get()
            .map_err(|_| AuthError::AuthenticateFail)?;

        let results: Vec<PrivateUserData> = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(&data.email))
            .select(PrivateUserData::as_select())
            .limit(1)
            .load::<PrivateUserData>(&mut connection)
            .map_err(|_| AuthError::AuthenticateFail)?;

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
            _ => Err(AuthError::AuthenticateFail),
        }
    }

    pub fn register_by_credentials(&self, data: &Credentials) -> Result<(), AuthError> {
        if data.is_valid() == false {
            return Err(AuthError::CredentialsInvalid);
        }
        let new_user = NewUser {
            email: data.email.to_owned(),
            password: Some(
                self.hash
                    .get_ref()
                    .hash_password(&data.password)
                    .map_err(|_| AuthError::HashingPasswordFail)?,
            ),
        };

        let mut connection = self
            .db_pool
            .get_ref()
            .get()
            .map_err(|_| AuthError::DbConnectionFail)?;

        diesel::insert_into(crate::schema::users::table)
            .values(new_user)
            .execute(&mut connection)
            .map_err(|e: Error| match &e {
                Error::DatabaseError(kind, _) => match &kind {
                    DatabaseErrorKind::UniqueViolation => AuthError::DuplicateEmail,
                    _ => AuthError::InsertNewUserFail,
                },
                _ => AuthError::InsertNewUserFail,
            })?;
        Ok(())
    }

    pub fn logout_from_session(&self, session: &Session) {
        self.remove_user_id_from_session(session);
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
