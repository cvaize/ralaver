use crate::app::models::user::AuthUser;
use crate::app::models::user::User;
use crate::db_connection::DbPool;
use actix_session::{Session};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use garde::Validate;
use serde::de::StdError;
use serde_derive::Deserialize;
use std::fmt;

pub static AUTH_USER_ID_KEY: &str = "app.auth.user.id";

pub struct Auth;

#[derive(Validate, Deserialize, Debug)]
pub struct Credentials {
    #[garde(required, inner(length(min = 1, max = 255)))]
    email: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    password: Option<String>,
}

impl Auth {
    pub fn insert_user_id_into_session(
        session: &Session,
        user_id: u64,
    ) -> Result<(), impl StdError + fmt::Display> {
        session.insert(AUTH_USER_ID_KEY, user_id)
    }

    pub fn get_user_id_from_session(
        session: &Session,
    ) -> Result<Option<u64>, impl StdError + fmt::Display> {
        session.get::<u64>(AUTH_USER_ID_KEY)
    }

    pub fn authenticate_from_session(
        db_pool: &DbPool,
        session: &Session,
    ) -> Result<User, UserIsNotAuthenticated> {
        let user_id = Auth::get_user_id_from_session(session)
            .map_err(|_| UserIsNotAuthenticated)?;

        match user_id {
            Some(id) => {
                let mut connection = db_pool.get()
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
    pub fn authenticate(
        db_pool: &DbPool,
        data: &Credentials,
    ) -> Result<u64, UserIsNotAuthenticated> {
        let id: Option<u64> = match data.email.to_owned() {
            Some(data_email) => {
                let mut connection = db_pool.get()
                    .map_err(|_| UserIsNotAuthenticated)?;

                let results: Vec<AuthUser> = crate::schema::users::dsl::users
                    .filter(crate::schema::users::email.eq(data_email))
                    .select(AuthUser::as_select())
                    .limit(1)
                    .load::<AuthUser>(&mut connection)
                    .map_err(|_| UserIsNotAuthenticated)?;

                let result: Option<&AuthUser> = results.get(0);

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
}


#[derive(Debug, Clone, Copy)]
pub struct UserIsNotAuthenticated;