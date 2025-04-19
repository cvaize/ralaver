use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::{error, Error, FromRequest, HttpMessage, HttpRequest};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInner(String);

impl Session {
    pub fn new(id: String, user_id: u64) -> Self {
        Self { id, user_id }
    }
}

impl FromRequest for Session {
    type Error = Error;
    type Future = Ready<Result<Session, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let extensions = &mut req.extensions_mut();
        if let Some(s) = extensions.get::<Session>() {
            return ready(Ok(s.clone()));
        }

        ready(Err(error::ErrorInternalServerError(
            "Session error in Session::from_request",
        )))
    }
}
