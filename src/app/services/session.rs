use crate::{Config, KeyValueService, RandomService, Session};
use actix_web::web::Data;
use strum_macros::Display;
use strum_macros::EnumString;


#[derive(Debug, Clone)]
pub struct SessionService {
    config: Data<Config>,
    key_value_service: Data<KeyValueService>,
    random_service: Data<RandomService>,
}

static SESSION_KEY: &str = "session";
static SESSION_ID_KEY: &str = "session_id";
static SESSION_USER_ID_KEY: &str = "user_id";
static SESSION_DATA_KEY: &str = "data";

impl SessionService {
    pub fn new(
        config: Data<Config>,
        key_value_service: Data<KeyValueService>,
        random_service: Data<RandomService>,
    ) -> Self {
        Self {
            config,
            key_value_service,
            random_service,
        }
    }

    fn make_session_key_to_id_key(&self, session_key: &str) -> String {
        format!("{}.{}.{}", SESSION_KEY, SESSION_ID_KEY, session_key)
    }

    pub fn make_session_data_key(&self, session: &Session, value_name: &str) -> String {
        format!(
            "{}.{}.{}.{}",
            SESSION_KEY, SESSION_DATA_KEY, &session.id, value_name
        )
    }

    fn make_session_data_key_(&self, session_id: &str, value_name: &str) -> String {
        format!(
            "{}.{}.{}.{}",
            SESSION_KEY, SESSION_DATA_KEY, session_id, value_name
        )
    }

    pub fn save_session(&self, session: &Session) -> Result<(), SessionServiceError> {
        let key_value_service = self.key_value_service.get_ref();
        let config = self.config.get_ref();

        key_value_service
            .set_ex(
                self.make_session_data_key_(&session.id, SESSION_USER_ID_KEY),
                session.user_id,
                config.session.expires,
            )
            .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

        Ok(())
    }

    pub fn delete_session(&self, session: &Session) -> Result<(), SessionServiceError> {
        let key_value_service = self.key_value_service.get_ref();

        // TODO: Удалить все записи с session.id
        key_value_service
            .del(self.make_session_data_key_(&session.id, SESSION_USER_ID_KEY))
            .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

        Ok(())
    }

    pub fn renew(&self, key: Option<String>) -> Result<(String, Session), SessionServiceError> {
        let key_value_service = self.key_value_service.get_ref();
        let mut key_value_conn = key_value_service.get_connection()
            .map_err(|_| SessionServiceError::KeyValueServiceFail)?;
        let random_service = self.random_service.get_ref();
        let config = self.config.get_ref();
        let mut user_id: u64 = 0;

        let mut id: Option<String> = match &key {
            // If there are problems with cookies, it is worth checking the comparison of string lengths in this place
            Some(key) => match key.len() == config.session.key_length {
                true => key_value_conn
                    .get(self.make_session_key_to_id_key(key))
                    .map_err(|_| SessionServiceError::KeyValueServiceFail)?,
                false => None,
            },
            None => None,
        };

        if let Some(id_) = &id {
            let saved_user_id: Option<u64> = key_value_conn
                .get_ex(
                    self.make_session_data_key_(id_, SESSION_USER_ID_KEY),
                    config.session.expires,
                )
                .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

            if let Some(saved_user_id) = saved_user_id {
                user_id = saved_user_id;
            } else {
                id = None;
            }
        }

        if id.is_none() {
            for _ in 0..10 {
                let id_ = random_service.str(config.session.id_length);
                let value: Option<u64> = key_value_conn
                    .get(self.make_session_data_key_(&id_, SESSION_USER_ID_KEY))
                    .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

                if value.is_none() {
                    key_value_conn
                        .set_ex(
                            self.make_session_data_key_(&id_, SESSION_USER_ID_KEY),
                            user_id,
                            config.session.expires,
                        )
                        .map_err(|_| SessionServiceError::KeyValueServiceFail)?;
                    id = Some(id_);
                    break;
                }
            }
        }

        if id.is_none() {
            return Err(SessionServiceError::RenewIdFail);
        }
        let id: String = id.unwrap();

        let mut new_key: Option<String> = None;
        for _ in 0..10 {
            let key_ = random_service.str(config.session.key_length);
            let value: Option<String> = key_value_conn
                .get(self.make_session_key_to_id_key(&key_))
                .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

            if value.is_none() {
                new_key = Some(key_);
                break;
            }
        }

        if new_key.is_none() {
            return Err(SessionServiceError::RenewKeyFail);
        }
        let new_key: String = new_key.unwrap();

        key_value_conn
            .set_ex(
                self.make_session_key_to_id_key(&new_key),
                &id,
                config.session.expires,
            )
            .map_err(|_| SessionServiceError::KeyValueServiceFail)?;

        if let Some(key) = key {
            key_value_conn
                .expire(
                    self.make_session_key_to_id_key(&key),
                    config.session.old_expires as i64,
                )
                .map_err(|_| SessionServiceError::KeyValueServiceFail)?;
        }

        Ok((new_key, Session::new(id, user_id)))
    }
}

// TODO: Продумать таймер, чтобы не обновлять [уникальный id(200 символов)] 10 минут или исключить статику (/css/app.css, /js/app.js) из установки ключа

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum SessionServiceError {
    // InsertFail,
    // GetFail,
    KeyValueServiceFail,
    // DeleteFail,
    RenewIdFail,
    RenewKeyFail,
    RenewFail,
}
