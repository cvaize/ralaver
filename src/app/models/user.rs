use crate::AuthService;
use actix_session::{Session, SessionExt};
use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{error, Error, FromRequest, HttpRequest};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct PrivateUserData {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<User, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session: Session = req.get_session();

        let auth: Option<&Data<AuthService>> = req.app_data::<Data<AuthService>>();
        if auth.is_none() {
            return ready(Err(error::ErrorInternalServerError("AuthService error")));
        }
        let auth_service = auth.unwrap();

        let user = auth_service.authenticate_by_session(&session);
        if let Err(_) = user {
            return ready(Err(error::ErrorUnauthorized("Unauthorized")));
        }
        let user = user.unwrap();

        ready(Ok(user))
    }
}
