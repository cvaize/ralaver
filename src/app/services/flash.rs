use crate::{log_map_err, KeyValueService, Session, SessionService};
use actix_web::web::Data;
use actix_web::{error, Error};
use redis::{FromRedisValue, ToRedisArgs};
use strum_macros::{Display, EnumString};


pub struct FlashService {
    key_value_service: Data<KeyValueService>,
    session_service: Data<SessionService>,
}

impl FlashService {
    pub fn new(
        key_value_service: Data<KeyValueService>,
        session_service: Data<SessionService>,
    ) -> Self {
        Self {
            key_value_service,
            session_service,
        }
    }

    pub fn save<V: ToRedisArgs>(&self, session: &Session, key: &str, data: V) -> Result<(), FlashServiceError> {
        let session_service = self.session_service.get_ref();
        let key_value_service = self.key_value_service.get_ref();
        let key = session_service.make_session_data_key(&session, key);

        key_value_service
            .set_ex(key, data, 600)
            .map_err(log_map_err!(
                FlashServiceError::KeyValueServiceFail,
                "FlashService::save"
            ))?;

        Ok(())
    }

    pub fn save_throw_http<V: ToRedisArgs>(&self, session: &Session, key: &str, data: V) -> Result<(), Error> {
        self.save(session, key, data)
            .map_err(log_map_err!(
                error::ErrorInternalServerError("FlashService error"),
                "FlashService::save_throw_http"
            ))
    }

    pub fn all<V: FromRedisValue>(&self, session: &Session, key: &str) -> Result<Option<V>, FlashServiceError> {
        let session_service = self.session_service.get_ref();
        let key_value_service = self.key_value_service.get_ref();
        let key = session_service.make_session_data_key(&session, key);

        let data = key_value_service.get_del(&key).map_err(log_map_err!(
            FlashServiceError::KeyValueServiceFail,
            "FlashService::all"
        ))?;
        Ok(data)
    }

    pub fn all_throw_http<V: FromRedisValue>(&self, session: &Session, key: &str) -> Result<Option<V>, Error> {
        self.all(&session, key).map_err(log_map_err!(
            error::ErrorInternalServerError("FlashService error"),
            "FlashService::all_throw_http"
        ))
    }

    pub fn delete(&self, session: &Session, key: &str) -> Result<(), FlashServiceError> {
        let session_service = self.session_service.get_ref();
        let key_value_service = self.key_value_service.get_ref();
        let key = session_service.make_session_data_key(&session, key);

        let data = key_value_service.del(&key).map_err(log_map_err!(
            FlashServiceError::KeyValueServiceFail,
            "FlashService::delete"
        ))?;
        Ok(data)
    }

    pub fn delete_throw_http(&self, session: &Session, key: &str) -> Result<(), Error> {
        self.delete(&session, key).map_err(log_map_err!(
            error::ErrorInternalServerError("FlashService error"),
            "FlashService::delete_throw_http"
        ))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum FlashServiceError {
    KeyValueServiceFail,
    DbConnectionFail,
}
