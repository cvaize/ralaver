use crate::app::models::user::AuthUser;
use crate::app::models::user::User;
use crate::db_connection::DbPool;
use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{error, Error, FromRequest, HttpRequest};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use garde::Validate;
use serde_derive::Deserialize;
use std::ops::Deref;

static USER_ID_KEY: &str = "app.auth.user.id";
pub static AUTHENTICATED_REDIRECT_TO: &str = "/";
pub static NOT_AUTHENTICATED_REDIRECT_TO: &str = "/login";

pub struct Auth {
    pub session: Session,
    pub db_pool: DbPool,
}

#[derive(Validate, Deserialize, Debug)]
pub struct Credentials {
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub email: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct UserIsNotAuthenticated;

impl Auth {
    pub fn new(session: Session, db_pool: DbPool) -> Self {
        Self { session, db_pool }
    }
    pub fn insert_user_id_into_session(&self, user_id: u64) -> Result<(), SessionInsertError> {
        self.session.insert(USER_ID_KEY, user_id)
    }

    pub fn get_user_id_from_session(&self) -> Result<Option<u64>, SessionGetError> {
        self.session.get::<u64>(USER_ID_KEY)
    }

    pub fn authenticate_from_session(&self) -> Result<User, UserIsNotAuthenticated> {
        let user_id = self
            .get_user_id_from_session()
            .map_err(|_| UserIsNotAuthenticated)?;

        match user_id {
            Some(id) => {
                let mut connection = self.db_pool.get().map_err(|_| UserIsNotAuthenticated)?;

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
    pub fn authenticate(&self, data: &Credentials) -> Result<u64, UserIsNotAuthenticated> {
        let id: Option<u64> = match data.email.to_owned() {
            Some(data_email) => {
                let mut connection = self.db_pool.get().map_err(|_| UserIsNotAuthenticated)?;

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

impl FromRequest for Auth {
    type Error = Error;
    type Future = Ready<Result<Auth, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session: Session = req.get_session();

        let db_pool: Option<&Data<DbPool>> = req.app_data::<Data<DbPool>>();
        if db_pool.is_none() {
            return ready(Err(error::ErrorInternalServerError("Db error")));
        }
        let db_pool = db_pool.unwrap().deref().deref().to_owned();

        let auth = Auth::new(session, db_pool);
        ready(Ok(auth))
    }
}
