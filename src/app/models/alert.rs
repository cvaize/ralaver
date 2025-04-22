use crate::{model_redis_impl, Translator};
use serde_bare;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub static ALERTS_KEY: &str = "alerts";

#[derive(Serialize, Deserialize, Debug)]
pub struct Alert {
    pub style: String,
    pub content: String,
}

impl Alert {
    pub fn new(style: String, content: String) -> Self {
        Self { style, content }
    }
    pub fn info(content: String) -> Self {
        Self::new("info".to_string(), content)
    }
    pub fn success(content: String) -> Self {
        Self::new("success".to_string(), content)
    }
    pub fn warning(content: String) -> Self {
        Self::new("warning".to_string(), content)
    }
    pub fn error(content: String) -> Self {
        Self::new("error".to_string(), content)
    }
    pub fn from_variant(translator: &Translator, variant: &AlertVariant) -> Self {
        match variant {
            _ => Self::success(translator.simple(variant.get_message_key())),
        }
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum AlertVariant {
    LoginSuccess,
    LogoutSuccess,
}

impl AlertVariant {
    pub fn get_message_key(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "auth.alert.login.success",
            Self::LogoutSuccess => "auth.alert.logout.success",
        }
    }
}

model_redis_impl!(Alert);
