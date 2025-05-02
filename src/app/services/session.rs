use crate::{Config, HashService, RandomService, WebAuthServiceError};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
use std::borrow::Cow;

pub static CSRF_ERROR_MESSAGE: &str = "CSRF token mismatch.";

#[derive(Debug, Clone)]
pub struct Session(Option<String>, String);

impl Session {
    pub fn new(old_token: Option<String>, new_token: String) -> Self {
        Self(old_token, new_token)
    }
    pub fn get_old_token(&self) -> Option<&String> {
        self.0.as_ref()
    }
    pub fn get_new_token(&self) -> &String {
        &self.1
    }
}

pub struct SessionService {
    config: Data<Config>,
    random_service: Data<RandomService>,
    hash_service: Data<HashService>,
}

impl SessionService {
    pub fn new(
        config: Data<Config>,
        random_service: Data<RandomService>,
        hash_service: Data<HashService>,
    ) -> Self {
        Self {
            config,
            random_service,
            hash_service,
        }
    }

    pub fn new_session_from_req(&self, req: &HttpRequest) -> Session {
        let cookie = req.cookie(&self.config.session.cookie_key);
        let mut old_token: Option<String> = None;
        if let Some(cookie) = cookie {
            let value: String = cookie.value().to_string();
            if value.len() == 32 {
                old_token = Some(value);
            }
        }
        let random_service = self.random_service.get_ref();
        let new_token: String = random_service.str(32);

        Session::new(old_token, new_token)
    }

    fn new_csrf_from_token(&self, token: &str) -> String {
        let config = self.config.get_ref();
        let hash_service = self.hash_service.get_ref();
        let mut csrf = token.to_owned();
        csrf.push_str(&config.app.key);
        hash_service.hex_hash(csrf)
    }

    pub fn new_csrf(&self, session: &Session) -> String {
        self.new_csrf_from_token(session.get_new_token())
    }

    pub fn check_csrf(&self, session: &Session, token: &str) -> bool {
        match session.get_old_token() {
            Some(old_token) => self.new_csrf_from_token(old_token).eq(token),
            _ => false,
        }
    }

    pub fn check_csrf_throw_http(
        &self,
        session: &Session,
        token: &Option<String>,
    ) -> Result<bool, Error> {
        if token.is_none() {
            return Err(error::ErrorForbidden(CSRF_ERROR_MESSAGE));
        }
        let token = token.as_ref().unwrap();

        let is = self.check_csrf(session, token);
        if is {
            Ok(is)
        } else {
            Err(error::ErrorForbidden(CSRF_ERROR_MESSAGE))
        }
    }

    fn make_cookie_<'a, V>(&'a self, token: V, max_age: u64) -> Cookie<'a>
    where
        V: Into<Cow<'a, str>>,
    {
        let config = self.config.get_ref();
        let mut cookie = Cookie::build(&config.session.cookie_key, token)
            .path(&config.session.cookie_path)
            .http_only(config.session.cookie_http_only)
            .secure(config.session.cookie_secure)
            .max_age(Duration::seconds(max_age as i64));

        if config.session.cookie_domain != "" {
            cookie = cookie.domain(&config.session.cookie_domain);
        }

        cookie.finish()
    }

    pub fn make_cookie<'a>(&'a self, session: &'a Session) -> Result<Cookie<'a>, WebAuthServiceError> {
        let config = self.config.get_ref();
        Ok(self.make_cookie_(session.get_new_token(), config.session.cookie_expires))
    }

    pub fn make_cookie_throw_http<'a>(&'a self, session: &'a Session) -> Result<Cookie<'a>, Error> {
        self.make_cookie(session).map_err(|e| {
            log::error!("SessionService::make_cookie_throw_http - {e}");
            return error::ErrorInternalServerError("SessionService error");
        })
    }

    pub fn make_clear_cookie(&self) -> Cookie {
        self.make_cookie_("", 0)
    }
}
