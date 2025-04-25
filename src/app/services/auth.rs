use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{
    log_map_err, Config, CryptService, KeyValueService, KeyValueServiceError,
    NewUser, PrivateUserData, RandomService, User, UserService,
};
use crate::{HashService, MysqlPool};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
#[allow(unused_imports)]
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde_derive::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::{Display, EnumString};

static RESET_PASSWORD_CODE_KEY: &str = "reset_password.code";

pub struct AuthService {
    #[allow(dead_code)]
    config: Data<Config>,
    db_pool: Data<MysqlPool>,
    hash: Data<HashService>,
    key_value_service: Data<KeyValueService>,
    user_service: Data<UserService>,
    random_service: Data<RandomService>,
    crypt_service: Data<CryptService>,
}

#[derive(Debug, Clone)]
pub struct AuthToken(u64, u64, String);

impl AuthToken {
    pub fn new(user_id: u64, token_id: u64, token_value: String) -> Self {
        AuthToken(user_id, token_id, token_value)
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
}

impl AuthService {
    pub fn new(
        config: Data<Config>,
        db_pool: Data<MysqlPool>,
        hash: Data<HashService>,
        key_value_service: Data<KeyValueService>,
        user_service: Data<UserService>,
        random_service: Data<RandomService>,
        crypt_service: Data<CryptService>,
    ) -> Self {
        Self {
            config,
            db_pool,
            hash,
            key_value_service,
            user_service,
            random_service,
            crypt_service,
        }
    }

    pub fn encrypt_auth_token(&self, auth_token: &AuthToken) -> Result<String, AuthServiceError> {
        let mut token: String = "".to_string();
        token.push_str(auth_token.0.to_string().as_str());
        token.push_str("-");
        token.push_str(auth_token.1.to_string().as_str());
        token.push_str("-");
        token.push_str(auth_token.2.as_str());
        self.crypt_service
            .get_ref()
            .encrypt_string(&token)
            .map_err(log_map_err!(
                AuthServiceError::Fail,
                "AuthService::encrypt_auth_token"
            ))
    }

    pub fn decrypt_auth_token(&self, encrypted_token: &str) -> Result<AuthToken, AuthServiceError> {
        let token = self
            .crypt_service
            .get_ref()
            .decrypt_string(encrypted_token)
            .map_err(log_map_err!(
                AuthServiceError::Fail,
                "AuthService::decrypt_auth_token"
            ))?;
        let s: Vec<&str> = token.split("-").collect();
        if s.len() != 3 {
            return Err(AuthServiceError::Fail);
        }
        let s1: u64 = s.get(0).unwrap().parse().map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::decrypt_auth_token"
        ))?;
        let s2: u64 = s.get(1).unwrap().parse().map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::decrypt_auth_token"
        ))?;
        let s3: String = s.get(2).unwrap().to_string();
        Ok(AuthToken::new(s1, s2, s3))
    }

    pub fn get_auth_token_from_request<'c>(&self, req: &HttpRequest) -> Option<AuthToken> {
        let config = self.config.get_ref();
        let token = req.cookie(&config.auth.token_cookie_key);
        if token.is_none() {
            return None;
        }
        let token = self.decrypt_auth_token(token.unwrap().value());
        if token.is_err() {
            return None;
        }
        Some(token.unwrap())
    }

    fn make_auth_token_cookie_<'b>(&'b self, token: String, max_age: u64) -> Cookie<'b> {
        let config = self.config.get_ref();
        let mut cookie = Cookie::build(&config.auth.token_cookie_key, token)
            .path(&config.auth.token_cookie_path)
            .http_only(config.auth.token_cookie_http_only)
            .secure(config.auth.token_cookie_secure)
            .max_age(Duration::seconds(max_age as i64));

        if config.auth.token_cookie_domain != "" {
            cookie = cookie.domain(&config.auth.token_cookie_domain);
        }

        cookie.finish()
    }

    pub fn make_auth_token_cookie(&self, token: &AuthToken) -> Result<Cookie, AuthServiceError> {
        let config = self.config.get_ref();
        let token = self.encrypt_auth_token(&token).map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::make_auth_token_cookie"
        ))?;
        Ok(self.make_auth_token_cookie_(token, config.auth.token_expires))
    }

    pub fn make_auth_token_cookie_throw_http(&self, token: &AuthToken) -> Result<Cookie, Error> {
        self.make_auth_token_cookie(token).map_err(log_map_err!(
            error::ErrorInternalServerError("AuthService error"),
            "AuthService::make_auth_token_cookie_throw_http"
        ))
    }

    pub fn make_auth_token_clear_cookie(&self) -> Cookie {
        self.make_auth_token_cookie_("".to_string(), 0)
    }

    pub fn generate_auth_token(&self, user_id: u64) -> AuthToken {
        let config = self.config.get_ref();
        let random_service = self.random_service.get_ref();
        let token: String = random_service.str(config.auth.token_length);
        let token_id: u64 = random_service.range(u64::MIN..=u64::MAX);
        AuthToken::new(user_id, token_id, token)
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

    pub fn save_auth_token(&self, token: &AuthToken) -> Result<(), AuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(log_map_err!(
                AuthServiceError::Fail,
                "AuthService::save_auth_token"
            ))?
            .as_secs()
            + config.auth.old_token_expires;

        let mut con = key_value_service.get_connection().map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::save_auth_token"
        ))?;

        con.set_ex(
            self.get_token_value_key(&token),
            token.get_token_value(),
            config.auth.token_expires,
        )
        .map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::save_auth_token"
        ))?;

        con.set_ex(
            self.get_token_expires_key(&token),
            expires,
            config.auth.token_expires,
        )
        .map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::save_auth_token"
        ))?;

        Ok(())
    }

    pub fn expire_auth_token(&self, token: &AuthToken) -> Result<(), AuthServiceError> {
        let config = self.config.get_ref();
        let key_value_service = self.key_value_service.get_ref();

        let mut con = key_value_service.get_connection().map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::expire_auth_token"
        ))?;

        con.expire(
            self.get_token_value_key(&token),
            config.auth.old_token_expires as i64,
        )
        .map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::expire_auth_token"
        ))?;

        con.expire(
            self.get_token_value_key(&token),
            config.auth.old_token_expires as i64,
        )
        .map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::expire_auth_token"
        ))?;

        Ok(())
    }

    pub fn login_by_auth_token(
        &self,
        token: &AuthToken,
    ) -> Result<(User, AuthToken), AuthServiceError> {
        let key_value_service = self.key_value_service.get_ref();

        let mut con = key_value_service.get_connection().map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::login_by_auth_token"
        ))?;

        let token_value: Option<String> =
            con.get(self.get_token_value_key(&token))
                .map_err(log_map_err!(
                    AuthServiceError::Fail,
                    "AuthService::login_by_auth_token"
                ))?;

        if token_value.is_none() {
            return Err(AuthServiceError::Fail);
        }

        let token_expires: Option<u64> =
            con.get(self.get_token_expires_key(&token))
                .map_err(log_map_err!(
                    AuthServiceError::Fail,
                    "AuthService::login_by_auth_token"
                ))?;

        if token_expires.is_none() {
            return Err(AuthServiceError::Fail);
        }
        let token_value = token_value.unwrap();

        if token_value != token.get_token_value() {
            return Err(AuthServiceError::Fail);
        }
        let token_expires = token_expires.unwrap();

        // Тут токен уже подтверждён и можно получить пользователя
        let user_service = self.user_service.get_ref();
        let user_id = token.get_user_id();
        let user = user_service.first_by_id(user_id).map_err(log_map_err!(
            AuthServiceError::Fail,
            "AuthService::login_by_auth_token"
        ))?;

        // Нужно сгенерировать новый токен
        let mut is_need_new_token = token_expires == 1;

        if !is_need_new_token {
            let expires = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(log_map_err!(
                    AuthServiceError::Fail,
                    "AuthService::login_by_auth_token"
                ))?
                .as_secs();

            is_need_new_token = token_expires < expires;
        }

        // Нужно пометить старый токен на удаление
        let is_expire_old_token = is_need_new_token && token_expires != 1;

        if is_expire_old_token {
            self.expire_auth_token(token).map_err(log_map_err!(
                AuthServiceError::Fail,
                "AuthService::login_by_auth_token"
            ))?
        }

        let token: AuthToken = if is_need_new_token {
            let token = self.generate_auth_token(user_id);
            self.save_auth_token(&token).map_err(log_map_err!(
                AuthServiceError::Fail,
                "AuthService::login_by_auth_token"
            ))?;
            token
        } else {
            token.clone()
        };

        Ok((user, token))
    }

    pub fn login_by_req(&self, req: &HttpRequest) -> Result<(User, AuthToken), AuthServiceError> {
        let auth_token = self.get_auth_token_from_request(req);

        if auth_token.is_none() {
            return Err(AuthServiceError::Fail);
        }
        let auth_token = auth_token.unwrap();
        self.login_by_auth_token(&auth_token)
    }

    pub fn logout_by_auth_token(&self, auth_token: &AuthToken) -> Result<(), AuthServiceError> {
        self.expire_auth_token(auth_token)?;
        Ok(())
    }

    pub fn logout_by_req(&self, req: &HttpRequest) -> Result<(), AuthServiceError> {
        let auth_token = self.get_auth_token_from_request(req);
        if auth_token.is_none() {
            return Ok(());
        }
        let auth_token = auth_token.unwrap();

        self.logout_by_auth_token(&auth_token)?;
        Ok(())
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn login_by_credentials(&self, data: &Credentials) -> Result<u64, AuthServiceError> {
        if data.is_valid() == false {
            return Err(AuthServiceError::Fail);
        }

        let mut connection = self.db_pool.get_ref().get().map_err(log_map_err!(
            AuthServiceError::DbConnectionFail,
            "AuthService::login_by_credentials"
        ))?;

        let user: PrivateUserData = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(&data.email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .map_err(|e| {
                if e.to_string() != "Record not found" {
                    log::error!(
                        "{}",
                        format!(
                            "AuthService::login_by_credentials - {} - {:}",
                            data.email, e
                        )
                        .as_str(),
                    );
                }
                return AuthServiceError::Fail;
            })?;

        // Check auth
        let id: Option<u64> = match &user.password {
            Some(user_password_hash) => {
                if self
                    .hash
                    .get_ref()
                    .verify_password(&data.password, user_password_hash)
                {
                    Some(user.id)
                } else {
                    None
                }
            }
            _ => None,
        };

        match id {
            Some(id) => Ok(id),
            _ => Err(AuthServiceError::Fail),
        }
    }

    /// Search for a user by the provided credentials and return his id.
    pub fn login_by_password(
        &self,
        email: &String,
        password: &String,
    ) -> Result<u64, AuthServiceError> {
        let hash_service = self.hash.get_ref();
        let mut connection = self.db_pool.get_ref().get().map_err(log_map_err!(
            AuthServiceError::DbConnectionFail,
            "AuthService::login_by_password"
        ))?;

        let user: PrivateUserData = crate::schema::users::dsl::users
            .filter(crate::schema::users::email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .map_err(|e| {
                if e.to_string() != "Record not found" {
                    let message = format!("AuthService::login_by_password - {} - {:}", email, e);
                    log::error!("{}", message.as_str(),);
                }
                return AuthServiceError::Fail;
            })?;

        if user.password.is_none() {
            return Err(AuthServiceError::Fail);
        }
        let user_password_hash = user.password.unwrap();
        let is_verified = hash_service.verify_password(password, &user_password_hash);

        if is_verified {
            Ok(user.id)
        } else {
            Err(AuthServiceError::Fail)
        }
    }

    pub fn register_by_credentials(&self, data: &Credentials) -> Result<(), AuthServiceError> {
        if data.is_valid() == false {
            return Err(AuthServiceError::CredentialsInvalid);
        }
        let new_user = NewUser {
            email: data.email.to_owned(),
            password: Some(
                self.hash
                    .get_ref()
                    .hash_password(&data.password)
                    .map_err(|e| {
                        log::error!(
                            "{}",
                            format!(
                                "AuthService::register_by_credentials - {} - {:}",
                                data.password, e
                            )
                            .as_str(),
                        );
                        AuthServiceError::PasswordHashFail
                    })?,
            ),
        };

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::register_by_credentials - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        diesel::insert_into(crate::schema::users::table)
            .values(new_user)
            .execute(&mut connection)
            .map_err(|e: diesel::result::Error| match &e {
                diesel::result::Error::DatabaseError(kind, _) => match &kind {
                    DatabaseErrorKind::UniqueViolation => {
                        log::info!(
                            "{}",
                            format!(
                                "AuthService::register_by_credentials - {} - {:}",
                                &data.email, e
                            )
                            .as_str(),
                        );
                        AuthServiceError::DuplicateEmail
                    }
                    _ => {
                        log::error!(
                            "{}",
                            format!("AuthService::register_by_credentials - {:}", &e).as_str(),
                        );
                        AuthServiceError::InsertNewUserFail
                    }
                },
                _ => {
                    log::error!(
                        "{}",
                        format!("AuthService::register_by_credentials - {:}", &e).as_str()
                    );
                    AuthServiceError::InsertNewUserFail
                }
            })?;
        Ok(())
    }

    pub fn save_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<(), KeyValueServiceError> {
        let key = format!("{}:{}", RESET_PASSWORD_CODE_KEY, &email);

        self.key_value_service
            .get_ref()
            .set(&key, code)
            .map_err(|e| {
                log::error!(
                    "{}",
                    format!("AuthService::save_reset_password_code - {} - {:}", &key, &e).as_str(),
                );
                e
            })?;
        Ok(())
    }

    pub fn get_reset_password_code(
        &self,
        email: &str,
    ) -> Result<Option<String>, KeyValueServiceError> {
        let key = format!("{}:{}", RESET_PASSWORD_CODE_KEY, &email);

        let value: Option<String> = self.key_value_service.get_ref().get(&key).map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::get_reset_password_code - {} - {:}", &key, &e).as_str(),
            );
            e
        })?;
        Ok(value)
    }

    pub fn is_equal_reset_password_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<bool, KeyValueServiceError> {
        let stored_code: Option<String> = self.get_reset_password_code(email).map_err(|e| {
            log::error!(
                "{}",
                format!(
                    "AuthService::is_equal_reset_password_code - {} - {:}",
                    email, e
                )
                .as_str(),
            );
            e
        })?;
        match stored_code {
            Some(stored_code) => Ok(stored_code.eq(code)),
            _ => Ok(false),
        }
    }

    pub fn update_password_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), AuthServiceError> {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::password as dsl_password;
        use crate::schema::users::dsl::users as dsl_users;

        let hashed_password = self.hash.get_ref().hash_password(password).map_err(|e| {
            log::error!(
                "{}",
                format!(
                    "AuthService::update_password_by_email - {} - {:}",
                    &email, &e
                )
                .as_str(),
            );
            AuthServiceError::Fail
        })?;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::update_password_by_email - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        diesel::update(dsl_users.filter(dsl_email.eq(email)))
            .set(dsl_password.eq(hashed_password))
            .execute(&mut connection)
            .map_err(|e| {
                log::error!(
                    "{}",
                    format!(
                        "AuthService::update_password_by_email - {} - {:}",
                        &email, &e
                    )
                    .as_str(),
                );
                AuthServiceError::Fail
            })?;
        Ok(())
    }

    pub fn exists_user_by_email(&self, email: &str) -> Result<bool, AuthServiceError> {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        use diesel::dsl::exists;
        use diesel::select;

        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!(
                "{}",
                format!("AuthService::exists_user_by_email - {:}", &e).as_str()
            );
            AuthServiceError::DbConnectionFail
        })?;

        let email_exists: bool = select(exists(dsl_users.filter(dsl_email.eq(email))))
            .get_result(&mut connection)
            .map_err(|e| {
                log::error!(
                    "{}",
                    format!("AuthService::exists_user_by_email - {:}", &e).as_str()
                );
                AuthServiceError::Fail
            })?;

        Ok(email_exists)
    }
}

#[derive(Deserialize, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl Credentials {
    pub fn is_valid(&self) -> bool {
        Email::apply(&self.email) && MinMaxLengthString::apply(&self.password, 4, 255)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum AuthServiceError {
    RegisterFail,
    CredentialsInvalid,
    DbConnectionFail,
    DuplicateEmail,
    InsertNewUserFail,
    PasswordHashFail,
    LogoutFail,
    Fail,
}

#[cfg(test)]
mod tests {
    use crate::app::services::auth::AuthToken;
    #[allow(unused_imports)]
    use crate::{preparation, Credentials, PrivateUserData};
    #[allow(unused_imports)]
    use diesel::prelude::*;
    #[allow(unused_imports)]
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
    #[allow(unused_imports)]
    use tokio;

    #[tokio::test]
    async fn exists_user_by_email() {
        let (_, all_services) = preparation();

        assert_eq!(
            false,
            all_services
                .auth
                .exists_user_by_email("null@null.null")
                .unwrap()
        );
        assert_eq!(
            true,
            all_services
                .auth
                .exists_user_by_email("admin@admin.example")
                .unwrap()
        );
    }

    #[tokio::test]
    async fn update_password_by_email() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;

        let (all_connections, all_services) = preparation();

        let email = "admin@admin.example";

        let password = all_services.rand.get_ref().str(64);
        all_services
            .auth
            .update_password_by_email(email, &password)
            .unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        let user_password_hash = user.password.clone().unwrap();

        assert_eq!(
            true,
            all_services
                .hash
                .get_ref()
                .verify_password(&password, &user_password_hash)
        );

        let password = all_services.rand.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .hash
                .get_ref()
                .verify_password(&password, &user_password_hash)
        );
    }

    #[tokio::test]
    async fn reset_password_code() {
        let (_, all_services) = preparation();

        let email = "admin@admin.example";
        let code = all_services.rand.get_ref().str(64);
        all_services
            .auth
            .save_reset_password_code(email, &code)
            .unwrap();

        let saved_code = all_services
            .auth
            .get_reset_password_code(email)
            .unwrap()
            .unwrap();
        assert_eq!(code, saved_code);
        assert_eq!(
            true,
            all_services
                .auth
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
        let code = all_services.rand.get_ref().str(64);
        assert_eq!(
            false,
            all_services
                .auth
                .is_equal_reset_password_code(email, &code)
                .unwrap()
        );
    }

    #[tokio::test]
    async fn login_by_credentials() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        let (all_connections, all_services) = preparation();

        let email = "admin@admin.example";
        let password = all_services.rand.get_ref().str(64);

        all_services
            .auth
            .update_password_by_email(email, &password)
            .unwrap();

        let cred = Credentials {
            email: email.to_owned(),
            password: password.to_owned(),
        };
        let user_id = all_services.auth.login_by_credentials(&cred).unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        assert_eq!(user.id, user_id);
    }

    #[tokio::test]
    async fn register_by_credentials() {
        use crate::schema::users::dsl::email as dsl_email;
        use crate::schema::users::dsl::users as dsl_users;
        let (all_connections, all_services) = preparation();

        let password = all_services.rand.get_ref().str(64);
        let email = format!("admin{}@admin.example", &password);

        let cred = Credentials {
            email: email.to_owned(),
            password: password.to_owned(),
        };
        all_services.auth.register_by_credentials(&cred).unwrap();

        let mut connection = all_connections.mysql.get_ref().get().unwrap();
        let user: PrivateUserData = dsl_users
            .filter(dsl_email.eq(email.to_owned()))
            .select(PrivateUserData::as_select())
            .first::<PrivateUserData>(&mut connection)
            .unwrap();

        assert_eq!(user.email, email);
    }

    #[tokio::test]
    async fn encrypt_auth_token() {
        let (_, all_services) = preparation();
        let auth = all_services.auth.get_ref();

        let auth_token = AuthToken(5, 6, "test".to_string());
        auth.encrypt_auth_token(&auth_token).unwrap();
    }

    #[tokio::test]
    async fn decrypt_auth_token() {
        let (_, all_services) = preparation();
        let auth = all_services.auth.get_ref();

        let auth_token = AuthToken(5, 6, "test".to_string());
        let s: String = auth.encrypt_auth_token(&auth_token).unwrap();
        let result: AuthToken = auth.decrypt_auth_token(&s).unwrap();
        assert_eq!(auth_token.get_token_id(), result.get_token_id());
        assert_eq!(auth_token.get_user_id(), result.get_user_id());
        assert_eq!(auth_token.get_token_value(), result.get_token_value());
    }
}
