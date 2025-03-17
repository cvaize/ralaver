use std::ops::Deref;
use crate::app::models::user::User;
use crate::db_connection::DbPool;
use actix_session::{Session, SessionGetError, SessionInsertError};
use actix_web::{error, Error};
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, ExpressionMethods};
use garde::Validate;
use serde_derive::Deserialize;

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
    pub fn insert_user_id_into_session(session: &Session, user_id: u64) -> Result<(), SessionInsertError> {
        session
            .insert(AUTH_USER_ID_KEY, user_id)
    }

    pub fn get_user_id_from_session(session: &Session) -> Result<Option<u64>, SessionGetError> {
        session
            .get::<u64>(AUTH_USER_ID_KEY)
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn authenticate(db_pool: &DbPool, data: &Credentials) -> Result<u64, Error> {
        let data_email = data.email.to_owned().unwrap_or("none".to_string());

        let mut connection = db_pool
            .get()
            .map_err(|_| error::ErrorInternalServerError("User is not authorized"))?;

        let results: Vec<User> = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(data_email))
            .select(User::as_select())
            .limit(1)
            .load::<User>(&mut connection)
            .map_err(|_| error::ErrorInternalServerError("User is not authorized"))?;

        let result: Option<&User> = results.get(0);

        // Check auth
        let user: Option<&User> = match result {
            Some(user) => match &user.password {
                Some(user_password) => match &data.password {
                    Some(data_password) => {
                        // TODO: Реализовать проверку пароля
                        if user_password.trim() == data_password.trim() {
                            Some(user)
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        // User authed
        if user.is_some() {
            let user = user.unwrap();
            Ok(user.id)
        } else {
            Err(error::ErrorInternalServerError("User is not authorized"))
        }
    }
}
