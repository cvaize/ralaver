use crate::{Config, CryptService, HashService, KeyValueService, RandomService, User, UserService};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::{Display, EnumString};

pub static CSRF_ERROR_MESSAGE: &str = "CSRF token mismatch.";

#[derive(Debug, Clone)]
pub struct Session(u64, u64, String, u64, Option<String>);

impl Session {
    pub fn new(
        user_id: u64,
        token_id: u64,
        token_value: String,
        expires: u64,
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
    pub fn get_expires(&self) -> u64 {
        self.3
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
        token.push_str(session.get_expires().to_string().as_str());
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
        let expires: u64 = split.get(3).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_session - {e}");
            return WebAuthServiceError::Fail;
        })?;
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
            return error::ErrorInternalServerError("WebAuthService error");
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
        let expires: u64 = config.auth.cookie.token_expires;
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

    pub fn get_token_expires_key(&self, token: &Session) -> String {
        let mut key = "auth.".to_string();
        key.push_str(token.get_user_id().to_string().as_str());
        key.push_str(".tokens.");
        key.push_str(token.get_token_id().to_string().as_str());
        key.push_str(".expires");
        key
    }

    fn make_store_data(&self, token_value: &str, expires: u64) -> String {
        let mut value = "".to_string();
        value.push_str(token_value);
        value.push('-');
        value.push_str(expires.to_string().as_str());
        value
    }

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

        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                log::error!("WebAuthService::save_session - {e}");
                return WebAuthServiceError::Fail;
            })?
            .as_secs()
            + config.auth.cookie.session_expires;

        key_value_service
            .set_ex(
                self.get_token_value_key(&token),
                self.make_store_data(token.get_token_value(), expires),
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

    pub fn login_by_session(
        &self,
        token: &Session,
    ) -> Result<(User, Session), WebAuthServiceError> {
        let key_value_service = self.key_value_service.get_ref();

        let value: Option<String> = key_value_service
            .get(self.get_token_value_key(&token))
            .map_err(|e| {
                log::error!("WebAuthService::login_by_session - {e}");
                return WebAuthServiceError::Fail;
            })?;

        if value.is_none() {
            return Err(WebAuthServiceError::Fail);
        }
        let value = value.unwrap();
        let (token_value, token_expires) = self.extract_store_data(&value).map_err(|e| {
            log::error!("WebAuthService::login_by_session - {e}");
            return WebAuthServiceError::Fail;
        })?;

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

        // Нужно сгенерировать новый токен, потому что этот старый токен помечен на удаление
        let mut is_need_new_token = token_expires == 1;

        if !is_need_new_token {
            let expires = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| {
                    log::error!("WebAuthService::login_by_session - {e}");
                    return WebAuthServiceError::Fail;
                })?
                .as_secs();

            // Дата сохранённая в token_expires меньше чем текущая дата expires
            is_need_new_token = token_expires < expires;
        }

        // Нужно пометить старый токен на удаление
        let is_expire_old_token = is_need_new_token && token_expires != 1;

        if is_expire_old_token {
            self.expire_session(token).map_err(|e| {
                log::error!("WebAuthService::login_by_session - {e}");
                return WebAuthServiceError::Fail;
            })?
        }

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
        hash_service.hex_hash(csrf)
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
    use crate::preparation;
    use crate::*;
    use test::Bencher;

    #[test]
    fn encrypt_session() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();

        let session = Session::new(5, 6, "test".to_string(), 100, None);
        auth.encrypt_session(&session).unwrap();
    }

    #[test]
    fn decrypt_session() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();

        let session = Session::new(5, 6, "test".to_string(), 100, None);
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
        let auth = all_services.web_auth.get_ref();

        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();
    }

    #[bench]
    fn bench_save_session(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let session = auth.generate_session(1);

        b.iter(|| {
            auth.save_session(&session).unwrap();
        });
    }

    #[bench]
    fn bench_expire_session(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();

        b.iter(|| {
            auth.expire_session(&session).unwrap();
        });
    }

    #[bench]
    fn bench_login_by_session(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let session = auth.generate_session(1);
        auth.save_session(&session).unwrap();

        b.iter(|| {
            auth.login_by_session(&session).unwrap();
        });
    }
}
