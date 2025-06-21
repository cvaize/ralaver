use crate::{Config, CryptService, HashService, KeyValueService, RandomService, User, UserService};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use std::borrow::Cow;
use std::ops::Add;
use strum_macros::{Display, EnumString};
use crate::helpers::DATE_TIME_FORMAT;

const FORMAT: &'static str = DATE_TIME_FORMAT;

pub const CSRF_ERROR_MESSAGE: &'static str = "CSRF token mismatch.";

#[derive(Debug, Clone)]
pub struct Session(u64, u64, String, DateTime<Utc>, Option<String>);

impl Session {
    pub fn new(
        user_id: u64,
        token_id: u64,
        token_value: String,
        expires: DateTime<Utc>,
        old_token_value: Option<String>,
    ) -> Self {
        Self(user_id, token_id, token_value, expires, old_token_value)
    }
    pub fn get_user_id(&self) -> u64 {
        self.0
    }
    pub fn get_token_id(&self) -> u64 {
        self.1
    }
    pub fn get_token_value(&self) -> &str {
        &self.2
    }
    pub fn get_expires(&self) -> &DateTime<Utc> {
        &self.3
    }
    pub fn get_old_token_value(&self) -> &Option<String> {
        &self.4
    }
    pub fn set_old_token_value(&mut self, v: Option<String>) {
        self.4 = v;
    }
}

pub struct WebAuthService {
    config: Data<Config>,
    crypt_service: Data<CryptService>,
    random_service: Data<RandomService>,
    key_value_service: Data<KeyValueService>,
    hash_service: Data<HashService>,
    user_service: Data<UserService>,
}

impl WebAuthService {
    pub fn new(
        config: Data<Config>,
        crypt_service: Data<CryptService>,
        random_service: Data<RandomService>,
        key_value_service: Data<KeyValueService>,
        hash_service: Data<HashService>,
        user_service: Data<UserService>,
    ) -> Self {
        Self {
            config,
            crypt_service,
            random_service,
            key_value_service,
            hash_service,
            user_service,
        }
    }

    pub fn encrypt_session(&self, session: &Session) -> Result<String, WebAuthServiceError> {
        let crypt_service = self.crypt_service.get_ref();
        let mut token: String = "".to_string();
        token.push_str(session.get_user_id().to_string().as_str());
        token.push('-');
        token.push_str(session.get_token_id().to_string().as_str());
        token.push('-');
        token.push_str(session.get_token_value());
        token.push('-');
        token.push_str(session.get_expires().format(FORMAT).to_string().as_str());
        crypt_service.encrypt_string(&token).map_err(|e| {
            log::error!("WebAuthService::encrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })
    }

    pub fn decrypt_session(&self, encrypted_token: &str) -> Result<Session, WebAuthServiceError> {
        let crypt_service = self.crypt_service.get_ref();
        let token = crypt_service.decrypt_string(encrypted_token).map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let split: Vec<&str> = token.split("-").collect();
        if split.len() != 4 {
            return Err(WebAuthServiceError::Fail);
        }
        let user_id: u64 = split.get(0).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let token_id: u64 = split.get(1).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let token_value: String = split.get(2).unwrap().to_string();
        let expires: String = split.get(3).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let expires = NaiveDateTime::parse_from_str(&expires, FORMAT).map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let expires: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(expires, Utc);
        Ok(Session::new(user_id, token_id, token_value, expires, None))
    }

    pub fn get_session_from_request(&self, req: &HttpRequest) -> Option<Session> {
        let config = self.config.get_ref();
        let token = req.cookie(&config.auth.cookie.cookie_key);
        if token.is_none() {
            return None;
        }
        let token = self.decrypt_session(token.unwrap().value());
        if token.is_err() {
            return None;
        }
        Some(token.unwrap())
    }

    fn make_cookie_<'a, V>(&'a self, token: V, max_age: u64) -> Cookie<'a>
    where
        V: Into<Cow<'a, str>>,
    {
        let config = self.config.get_ref();
        let mut cookie = Cookie::build(&config.auth.cookie.cookie_key, token)
            .path(&config.auth.cookie.cookie_path)
            .http_only(config.auth.cookie.cookie_http_only)
            .secure(config.auth.cookie.cookie_secure)
            .max_age(Duration::seconds(max_age as i64));

        if config.auth.cookie.cookie_domain != "" {
            cookie = cookie.domain(&config.auth.cookie.cookie_domain);
        }

        cookie.finish()
    }

    pub fn make_cookie(&self, token: &Session) -> Result<Cookie, WebAuthServiceError> {
        let config = self.config.get_ref();
        let session = self.encrypt_session(&token).map_err(|e| {
            log::error!("WebAuthService::make_cookie - {e}");
            return WebAuthServiceError::Fail;
        })?;
        Ok(self.make_cookie_(session, config.auth.cookie.token_expires))
    }

    pub fn make_cookie_throw_http(&self, token: &Session) -> Result<Cookie, Error> {
        self.make_cookie(token).map_err(|e| {
            log::error!("WebAuthService::make_cookie_throw_http - {e}");
            return error::ErrorInternalServerError("");
        })
    }

    pub fn make_clear_cookie(&self) -> Cookie {
        self.make_cookie_("", 0)
    }

    pub fn generate_session(&self, user_id: u64) -> Session {
        let config = self.config.get_ref();
        let random_service = self.random_service.get_ref();
        let token: String = random_service.str(config.auth.cookie.token_length);
        let token_id: u64 = random_service.range(u64::MIN..=u64::MAX);
        let expires: u64 = config.auth.cookie.session_expires;
        let expires: DateTime<Utc> = Utc::now().add(TimeDelta::seconds(expires as i64));
        Session::new(user_id, token_id, token, expires, None)
    }

    pub fn get_token_value_key(&self, token: &Session) -> String {
        let mut key = "auth.".to_string();
        key.push_str(token.get_user_id().to_string().as_str());
        key.push_str(".tokens.");
        key.push_str(token.get_token_id().to_string().as_str());
        key.push_str(".value");
        key
    }

    #[allow(dead_code)]
    fn make_store_data(&self, token_value: &str, expires: u64) -> String {
        let mut value = "".to_string();
        value.push_str(token_value);
        value.push('-');
        value.push_str(expires.to_string().as_str());
        value
    }

    #[allow(dead_code)]
    fn extract_store_data(&self, value: &str) -> Result<(String, u64), WebAuthServiceError> {
        let v: Vec<&str> = value.split("-").collect();
        let v0 = v.get(0);
        let v1 = v.get(1);
        if v0.is_none() || v1.is_none() {
            log::error!("WebAuthService::extract_store_data - {}", value);
            return Err(WebAuthServiceError::Fail);
        }
        Ok((
            v0.unwrap().to_string(),
            v1.unwrap().parse::<u64>().map_err(|e| {
                log::error!("WebAuthService::extract_store_data - {e}");
                return WebAuthServiceError::Fail;
            })?,
        ))
    }

    pub fn save_session(&self, token: &Session) -> Result<(), WebAuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        key_value_service
            .set_ex(
                self.get_token_value_key(&token),
                token.get_token_value(),
                config.auth.cookie.token_expires,
            )
            .map_err(|e| {
                log::error!("WebAuthService::save_session - {e}");
                return WebAuthServiceError::Fail;
            })?;

        Ok(())
    }

    pub fn expire_session(&self, token: &Session) -> Result<(), WebAuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        key_value_service
            .expire(
                self.get_token_value_key(&token),
                config.auth.cookie.session_expires as i64,
            )
            .map_err(|e| {
                log::error!("WebAuthService::expire_session - {e}");
                return WebAuthServiceError::Fail;
            })?;

        Ok(())
    }

    pub fn is_need_new_token(&self, token: &Session) -> bool {
        let expires = token.get_expires();
        let now = Utc::now();
        now.ge(expires)
    }

    pub fn login_by_session(
        &self,
        token: &Session,
    ) -> Result<(User, Session), WebAuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        let is_need_new_token = self.is_need_new_token(token);

        let token_expires = if is_need_new_token {
            config.auth.cookie.session_expires
        } else {
            config.auth.cookie.token_expires
        };
        let value: Option<String> = key_value_service
            .get_ex(self.get_token_value_key(&token), token_expires)
            .map_err(|e| {
                log::error!("WebAuthService::login_by_session - {e}");
                return WebAuthServiceError::Fail;
            })?;

        if value.is_none() {
            return Err(WebAuthServiceError::Fail);
        }
        let token_value = value.unwrap();

        if token_value != token.get_token_value() {
            return Err(WebAuthServiceError::Fail);
        }

        // Тут токен уже подтверждён и можно получить пользователя
        let user_service = self.user_service.get_ref();
        let user_id = token.get_user_id();
        let user = user_service.first_by_id(user_id).map_err(|e| {
            log::error!("WebAuthService::login_by_session - {e}");
            return WebAuthServiceError::Fail;
        })?;

        if user.is_none() {
            return Err(WebAuthServiceError::Fail);
        }

        let user = user.unwrap();

        let token: Session = if is_need_new_token {
            let mut new_token = self.generate_session(user_id);
            new_token.set_old_token_value(Some(token.get_token_value().to_owned()));
            self.save_session(&new_token).map_err(|e| {
                log::error!("WebAuthService::login_by_session - {e}");
                return WebAuthServiceError::Fail;
            })?;
            new_token
        } else {
            token.clone()
        };

        Ok((user, token))
    }

    pub fn login_by_req(&self, req: &HttpRequest) -> Result<(User, Session), WebAuthServiceError> {
        let session = self.get_session_from_request(req);

        if session.is_none() {
            return Err(WebAuthServiceError::Fail);
        }
        let session = session.unwrap();
        self.login_by_session(&session)
    }

    pub fn logout_by_session(&self, session: &Session) -> Result<(), WebAuthServiceError> {
        self.expire_session(session)?;
        Ok(())
    }

    pub fn logout_by_req(&self, req: &HttpRequest) -> Result<(), WebAuthServiceError> {
        let session = self.get_session_from_request(req);
        if session.is_none() {
            return Ok(());
        }
        let session = session.unwrap();

        self.logout_by_session(&session)?;
        Ok(())
    }

    fn new_csrf_from_token(&self, token: &str) -> String {
        let config = self.config.get_ref();
        let hash_service = self.hash_service.get_ref();
        let mut csrf = token.to_owned();
        csrf.push_str(&config.app.key);
        hash_service.hash(csrf)
    }

    pub fn new_csrf(&self, session: &Session) -> String {
        self.new_csrf_from_token(session.get_token_value())
    }

    pub fn check_csrf(&self, session: &Session, token: &str) -> bool {
        if let Some(old_token) = session.get_old_token_value() {
            if self.new_csrf_from_token(old_token).eq(token) {
                return true;
            }
        }
        let actual_token = session.get_token_value();
        if self.new_csrf_from_token(actual_token).eq(token) {
            return true;
        }
        false
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
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum WebAuthServiceError {
    Fail,
}

#[cfg(test)]
mod tests {
    use crate::app::services::web_auth::FORMAT;
    use crate::preparation;
    use crate::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use test::Bencher;

    fn get_now_date_time() -> DateTime<Utc> {
        let datetime: DateTime<Utc> = Utc::now();
        let datetime: String = datetime.format(FORMAT).to_string();
        let datetime: NaiveDateTime = NaiveDateTime::parse_from_str(&datetime, FORMAT).unwrap();
        DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc)
    }

    #[test]
    fn encrypt_session() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();

        let expires: DateTime<Utc> = get_now_date_time();
        let session = Session::new(5, 6, "test".to_string(), expires, None);
        auth.encrypt_session(&session).unwrap();
    }

    #[test]
    fn decrypt_session() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();

        let expires: DateTime<Utc> = get_now_date_time();
        let session = Session::new(5, 6, "test".to_string(), expires, None);
        let s: String = auth.encrypt_session(&session).unwrap();
        let result: Session = auth.decrypt_session(&s).unwrap();
        assert_eq!(session.get_token_id(), result.get_token_id());
        assert_eq!(session.get_user_id(), result.get_user_id());
        assert_eq!(session.get_token_value(), result.get_token_value());
        assert_eq!(session.get_expires(), result.get_expires());
    }

    #[test]
    fn save_session() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();

        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();
    }

    #[bench]
    fn bench_save_session(b: &mut Bencher) {
        // 47,360.80 ns/iter (+/- 6,764.60)
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();
        let session = auth.generate_session(1);

        b.iter(|| {
            auth.save_session(&session).unwrap();
        });
    }

    #[bench]
    fn bench_expire_session(b: &mut Bencher) {
        // 46,769.59 ns/iter (+/- 5,040.02)
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();
        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();

        b.iter(|| {
            auth.expire_session(&session).unwrap();
        });
    }

    #[bench]
    fn bench_login_by_session(b: &mut Bencher) {
        // 164,653.54 ns/iter (+/- 35,507.35)
        let (_, all_services) = preparation();
        let auth = all_services.web_auth_service.get_ref();
        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();

        b.iter(|| {
            auth.login_by_session(&session).unwrap();
        });
    }
}
