use actix_session::Session;
use serde_derive::{Deserialize, Serialize};

static SESSION_FLASH_DATA_KEY: &str = "app.session.flash_data.";
static SESSION_FLASH_DATA_COMMON_KEY: &str = "app.session.flash_data.common";

pub struct SessionFlashService<'a> {
    pub key: String,
    pub session: &'a Session,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionFlashData {
    pub alerts: Option<Vec<SessionFlashAlert>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum SessionFlashAlert {
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct SessionFlashServiceError;

pub trait SessionFlashDataTrait {
    fn empty() -> Self;
}

impl SessionFlashDataTrait for SessionFlashData {
    fn empty() -> Self {
        Self { alerts: None }
    }
}

impl<'a> SessionFlashService<'a> {
    pub fn new(session: &'a Session, key: Option<&str>) -> Self {
        let key: String = match key {
            Some(k) => format!("{}{}", SESSION_FLASH_DATA_KEY, k),
            None => SESSION_FLASH_DATA_COMMON_KEY.to_string()
        };
        Self { key, session }
    }

    pub fn save<T>(&self, data: &T) -> Result<(), SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::Serialize,
    {
        let json: String = serde_json::to_string(&data).map_err(|_| SessionFlashServiceError)?;
        self.session
            .insert(self.key.as_str(), json)
            .map_err(|_| SessionFlashServiceError)?;
        Ok(())
    }

    pub fn read<T>(&self) -> Result<T, SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::de::DeserializeOwned,
    {
        let result: Option<String> = self
            .session
            .get::<String>(self.key.as_str())
            .map_err(|_| SessionFlashServiceError)?;

        match result {
            Some(str) => {
                let flash_data: T =
                    serde_json::from_str(&str).map_err(|_| SessionFlashServiceError)?;
                Ok(flash_data)
            }
            _ => Ok(T::empty()),
        }
    }

    pub fn read_and_forget<T>(&self) -> Result<T, SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::de::DeserializeOwned,
    {
        let flash_data: T = self.read()?;
        self.session.remove(self.key.as_str());
        Ok(flash_data)
    }
}
