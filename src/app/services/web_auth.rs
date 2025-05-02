use crate::{Config, CryptService, KeyValueService, RandomService, User, UserService};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct AuthToken(u64, u64, String, u64);

impl AuthToken {
    pub fn new(user_id: u64, token_id: u64, token_value: String, expires: u64) -> Self {
        Self(user_id, token_id, token_value, expires)
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
}

pub struct WebAuthService {
    config: Data<Config>,
    crypt_service: Data<CryptService>,
    random_service: Data<RandomService>,
    key_value_service: Data<KeyValueService>,
    user_service: Data<UserService>,
}

impl WebAuthService {
    pub fn new(
        config: Data<Config>,
        crypt_service: Data<CryptService>,
        random_service: Data<RandomService>,
        key_value_service: Data<KeyValueService>,
        user_service: Data<UserService>,
    ) -> Self {
        Self {
            config,
            crypt_service,
            random_service,
            key_value_service,
            user_service,
        }
    }

    pub fn encrypt_auth_token(
        &self,
        auth_token: &AuthToken,
    ) -> Result<String, WebAuthServiceError> {
        let crypt_service = self.crypt_service.get_ref();
        let mut token: String = "".to_string();
        token.push_str(auth_token.get_user_id().to_string().as_str());
        token.push('-');
        token.push_str(auth_token.get_token_id().to_string().as_str());
        token.push('-');
        token.push_str(auth_token.get_token_value());
        token.push('-');
        token.push_str(auth_token.get_expires().to_string().as_str());
        crypt_service.encrypt_string(&token).map_err(|e| {
            log::error!("WebAuthService::encrypt_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })
    }

    pub fn decrypt_auth_token(
        &self,
        encrypted_token: &str,
    ) -> Result<AuthToken, WebAuthServiceError> {
        let crypt_service = self.crypt_service.get_ref();
        let token = crypt_service.decrypt_string(encrypted_token).map_err(|e| {
            log::error!("WebAuthService::decrypt_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let split: Vec<&str> = token.split("-").collect();
        if split.len() != 4 {
            return Err(WebAuthServiceError::Fail);
        }
        let user_id: u64 = split.get(0).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let token_id: u64 = split.get(1).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;
        let token_value: String = split.get(2).unwrap().to_string();
        let expires: u64 = split.get(3).unwrap().parse().map_err(|e| {
            log::error!("WebAuthService::decrypt_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;
        Ok(AuthToken::new(user_id, token_id, token_value, expires))
    }

    pub fn get_auth_token_from_request(&self, req: &HttpRequest) -> Option<AuthToken> {
        let config = self.config.get_ref();
        let token = req.cookie(&config.auth.cookie.cookie_key);
        if token.is_none() {
            return None;
        }
        let token = self.decrypt_auth_token(token.unwrap().value());
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

    pub fn make_cookie(&self, token: &AuthToken) -> Result<Cookie, WebAuthServiceError> {
        let config = self.config.get_ref();
        let auth_token = self.encrypt_auth_token(&token).map_err(|e| {
            log::error!("WebAuthService::make_cookie - {e}");
            return WebAuthServiceError::Fail;
        })?;
        Ok(self.make_cookie_(auth_token, config.auth.cookie.token_expires))
    }

    pub fn make_cookie_throw_http(&self, token: &AuthToken) -> Result<Cookie, Error> {
        self.make_cookie(token).map_err(|e| {
            log::error!("WebAuthService::make_cookie_throw_http - {e}");
            return error::ErrorInternalServerError("WebAuthService error");
        })
    }

    pub fn make_clear_cookie(&self) -> Cookie {
        self.make_cookie_("", 0)
    }

    pub fn generate_auth_token(&self, user_id: u64) -> AuthToken {
        let config = self.config.get_ref();
        let random_service = self.random_service.get_ref();
        let token: String = random_service.str(config.auth.cookie.token_length);
        let token_id: u64 = random_service.range(u64::MIN..=u64::MAX);
        let expires: u64 = config.auth.cookie.token_expires;
        AuthToken::new(user_id, token_id, token, expires)
    }

    pub fn get_token_value_key(&self, token: &AuthToken) -> String {
        let mut key = "auth.".to_string();
        key.push_str(token.get_user_id().to_string().as_str());
        key.push_str(".tokens.");
        key.push_str(token.get_token_id().to_string().as_str());
        key.push_str(".value");
        key
    }

    pub fn get_token_expires_key(&self, token: &AuthToken) -> String {
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

    pub fn save_auth_token(&self, token: &AuthToken) -> Result<(), WebAuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                log::error!("WebAuthService::save_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?
            .as_secs()
            + config.auth.cookie.old_token_expires;

        key_value_service
            .set_ex(
                self.get_token_value_key(&token),
                self.make_store_data(token.get_token_value(), expires),
                config.auth.cookie.token_expires,
            )
            .map_err(|e| {
                log::error!("WebAuthService::save_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?;

        Ok(())
    }

    pub fn expire_auth_token(&self, token: &AuthToken) -> Result<(), WebAuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        key_value_service
            .expire(
                self.get_token_value_key(&token),
                config.auth.cookie.old_token_expires as i64,
            )
            .map_err(|e| {
                log::error!("WebAuthService::expire_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?;

        Ok(())
    }

    pub fn login_by_auth_token(
        &self,
        token: &AuthToken,
    ) -> Result<(User, AuthToken), WebAuthServiceError> {
        let key_value_service = self.key_value_service.get_ref();

        let value: Option<String> = key_value_service
            .get(self.get_token_value_key(&token))
            .map_err(|e| {
                log::error!("WebAuthService::login_by_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?;

        if value.is_none() {
            return Err(WebAuthServiceError::Fail);
        }
        let value = value.unwrap();
        let (token_value, token_expires) = self.extract_store_data(&value).map_err(|e| {
            log::error!("WebAuthService::login_by_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;

        if token_value != token.get_token_value() {
            return Err(WebAuthServiceError::Fail);
        }

        // Тут токен уже подтверждён и можно получить пользователя
        let user_service = self.user_service.get_ref();
        let user_id = token.get_user_id();
        let user = user_service.first_by_id(user_id).map_err(|e| {
            log::error!("WebAuthService::login_by_auth_token - {e}");
            return WebAuthServiceError::Fail;
        })?;

        // Нужно сгенерировать новый токен
        let mut is_need_new_token = token_expires == 1;

        if !is_need_new_token {
            let expires = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| {
                    log::error!("WebAuthService::login_by_auth_token - {e}");
                    return WebAuthServiceError::Fail;
                })?
                .as_secs();

            is_need_new_token = token_expires < expires;
        }

        // Нужно пометить старый токен на удаление
        let is_expire_old_token = is_need_new_token && token_expires != 1;

        if is_expire_old_token {
            self.expire_auth_token(token).map_err(|e| {
                log::error!("WebAuthService::login_by_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?
        }

        let token: AuthToken = if is_need_new_token {
            let token = self.generate_auth_token(user_id);
            self.save_auth_token(&token).map_err(|e| {
                log::error!("WebAuthService::login_by_auth_token - {e}");
                return WebAuthServiceError::Fail;
            })?;
            token
        } else {
            token.clone()
        };

        Ok((user, token))
    }

    pub fn login_by_req(
        &self,
        req: &HttpRequest,
    ) -> Result<(User, AuthToken), WebAuthServiceError> {
        let auth_token = self.get_auth_token_from_request(req);

        if auth_token.is_none() {
            return Err(WebAuthServiceError::Fail);
        }
        let auth_token = auth_token.unwrap();
        self.login_by_auth_token(&auth_token)
    }

    pub fn logout_by_auth_token(&self, auth_token: &AuthToken) -> Result<(), WebAuthServiceError> {
        self.expire_auth_token(auth_token)?;
        Ok(())
    }

    pub fn logout_by_req(&self, req: &HttpRequest) -> Result<(), WebAuthServiceError> {
        let auth_token = self.get_auth_token_from_request(req);
        if auth_token.is_none() {
            return Ok(());
        }
        let auth_token = auth_token.unwrap();

        self.logout_by_auth_token(&auth_token)?;
        Ok(())
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
    fn encrypt_auth_token() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();

        let auth_token = AuthToken::new(5, 6, "test".to_string(), 100);
        auth.encrypt_auth_token(&auth_token).unwrap();
    }

    #[test]
    fn decrypt_auth_token() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();

        let auth_token = AuthToken::new(5, 6, "test".to_string(), 100);
        let s: String = auth.encrypt_auth_token(&auth_token).unwrap();
        let result: AuthToken = auth.decrypt_auth_token(&s).unwrap();
        assert_eq!(auth_token.get_token_id(), result.get_token_id());
        assert_eq!(auth_token.get_user_id(), result.get_user_id());
        assert_eq!(auth_token.get_token_value(), result.get_token_value());
        assert_eq!(auth_token.get_expires(), result.get_expires());
    }

    #[test]
    fn save_auth_token() {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();

        let auth_token = auth.generate_auth_token(1);
        auth.save_auth_token(&auth_token).unwrap();
    }

    #[bench]
    fn bench_save_auth_token(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let auth_token = auth.generate_auth_token(1);

        b.iter(|| {
            auth.save_auth_token(&auth_token).unwrap();
        });
    }

    #[bench]
    fn bench_expire_auth_token(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let auth_token = auth.generate_auth_token(1);
        auth.save_auth_token(&auth_token).unwrap();

        b.iter(|| {
            auth.expire_auth_token(&auth_token).unwrap();
        });
    }

    #[bench]
    fn bench_login_by_auth_token(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let auth = all_services.web_auth.get_ref();
        let auth_token = auth.generate_auth_token(1);
        auth.save_auth_token(&auth_token).unwrap();

        b.iter(|| {
            auth.login_by_auth_token(&auth_token).unwrap();
        });
    }
}
