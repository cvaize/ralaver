use crate::Config;
use actix_session::Session;
use actix_web::web::Data;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SessionService {
    #[allow(dead_code)]
    config: Config,
}

impl SessionService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn insert(
        &self,
        session: &Session,
        key: &str,
        data: &impl Serialize,
    ) -> Result<(), SaveSessionDataError> {
        session
            .insert(key, data)
            .map_err(|_| SaveSessionDataError)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(
        &self,
        session: &Session,
        key: &str,
    ) -> Result<Option<T>, GetSessionDataError> {
        let result: Option<T> = session
            .get::<T>(key)
            .map_err(|_| GetSessionDataError)?;

        Ok(result)
    }

    pub fn remove(&self, session: &Session, key: &str) {
        session.remove(key);
    }

    pub fn get_and_remove<T: DeserializeOwned>(
        &self,
        session: &Session,
        key: &str,
    ) -> Result<Option<T>, GetSessionDataError> {
        let data: Option<T> = self.get(session, key)?;
        self.remove(session, key);
        Ok(data)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SaveSessionDataError;
#[derive(Debug, Clone, Copy)]
pub struct GetSessionDataError;
