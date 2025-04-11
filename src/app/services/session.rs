use crate::{Config, LogService};
use actix_session::Session;
use actix_web::web::Data;
use serde::de::DeserializeOwned;
use serde::Serialize;
use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Debug, Clone)]
pub struct SessionService {
    #[allow(dead_code)]
    config: Config,
    log_service: Data<LogService>,
}

impl SessionService {
    pub fn new(config: Config, log_service: Data<LogService>) -> Self {
        Self {
            config,
            log_service,
        }
    }

    pub fn insert(
        &self,
        session: &Session,
        key: impl Into<String>,
        data: &impl Serialize,
    ) -> Result<(), SessionServiceError> {
        session.insert(key, data).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("SessionService::insert - {:}", &e).as_str());
            return SessionServiceError::InsertFail;
        })?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(
        &self,
        session: &Session,
        key: &str,
    ) -> Result<Option<T>, SessionServiceError> {
        let result: Option<T> = session.get::<T>(key).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("SessionService::get - {:}", &e).as_str());
            return SessionServiceError::GetFail;
        })?;

        Ok(result)
    }

    pub fn remove(&self, session: &Session, key: &str) {
        session.remove(key);
    }

    pub fn get_and_remove<T: DeserializeOwned>(
        &self,
        session: &Session,
        key: &str,
    ) -> Result<Option<T>, SessionServiceError> {
        let data: Option<T> = self.get(session, &key)?;
        self.remove(session, &key);
        Ok(data)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum SessionServiceError {
    InsertFail,
    GetFail,
}
