use crate::DbPool;
use crate::{Config, PrivateUserData, User};
use actix_session::{Session, SessionGetError, SessionInsertError};
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use garde::Validate;
use serde_derive::Deserialize;

#[derive(Debug)]
pub struct AuthService {
    config: Data<Config>,
    db_pool: Data<DbPool>,
}

#[derive(Debug, Clone, Copy)]
pub struct UserIsNotAuthenticated;

impl AuthService {
    pub fn new(config: Data<Config>, db_pool: Data<DbPool>) -> Self {
        Self { config, db_pool }
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

    pub fn authenticate_by_session(
        &self,
        session: &Session,
    ) -> Result<User, UserIsNotAuthenticated> {
        let user_id = self
            .get_user_id_from_session(session)
            .map_err(|_| UserIsNotAuthenticated)?;

        match user_id {
            Some(id) => {
                let mut connection = self
                    .db_pool
                    .get_ref()
                    .get()
                    .map_err(|_| UserIsNotAuthenticated)?;

                let user = crate::schema::users::dsl::users
                    .find(id)
                    .select(User::as_select())
                    .first(&mut connection)
                    .map_err(|_| UserIsNotAuthenticated)?;

                Ok(user)
            }
            _ => Err(UserIsNotAuthenticated),
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn authenticate_by_credentials(
        &self,
        data: &Credentials,
    ) -> Result<u64, UserIsNotAuthenticated> {
        let id: Option<u64> = match data.email.to_owned() {
            Some(data_email) => {
                let mut connection = self
                    .db_pool
                    .get_ref()
                    .get()
                    .map_err(|_| UserIsNotAuthenticated)?;

                let results: Vec<PrivateUserData> = crate::schema::users::dsl::users
                    .filter(crate::schema::users::email.eq(data_email))
                    .select(PrivateUserData::as_select())
                    .limit(1)
                    .load::<PrivateUserData>(&mut connection)
                    .map_err(|_| UserIsNotAuthenticated)?;

                let result: Option<&PrivateUserData> = results.get(0);

                // Check auth
                match result {
                    Some(user) => match &user.password {
                        Some(user_password) => match &data.password {
                            Some(data_password) => {
                                // TODO: Реализовать проверку пароля
                                if user_password.trim() == data_password.trim() {
                                    Some(user.id)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        };

        match id {
            Some(id) => Ok(id),
            _ => Err(UserIsNotAuthenticated),
        }
    }

    pub fn logout_from_session(&self, session: &Session) {
        self.remove_user_id_from_session(session);
    }
}

#[derive(Validate, Deserialize, Debug)]
pub struct Credentials {
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub email: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub password: Option<String>,
}
