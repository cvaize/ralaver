use crate::app::services::auth::Auth;
use crate::db_connection::DbPool;
use actix_session::Session;
use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{error, Error, FromRequest, HttpRequest};
use diesel::prelude::*;
use serde::Serialize;
use std::ops::Deref;

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
#[derive(Debug, Default, Serialize)]
pub struct User {
    pub id: u64,
    pub email: String,
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
#[derive(Debug, Default, Serialize)]
pub struct AuthUser {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<User, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let auth: Ready<Result<Auth, Error>> = Auth::from_request(req, payload);
        let auth = auth.into_inner();

        if let Err(e) = auth {
            return ready(Err(e));
        }
        let auth = auth.unwrap();

        let user = auth.authenticate_from_session();
        if let Err(_) = user {
            return ready(Err(error::ErrorUnauthorized("Unauthorized")));
        }
        let user = user.unwrap();

        ready(Ok(user))
    }
}
