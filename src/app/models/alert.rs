use crate::{model_redis_impl, TranslatorService};
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
    pub fn from_variant(translator_service: &TranslatorService, lang: &str, variant: &AlertVariant) -> Self {
        match variant {
            AlertVariant::ResetPasswordConfirmCodeNotEqual => {
                Self::error(translator_service.translate(&lang, variant.get_message_key()))
            }
            _ => Self::success(translator_service.translate(&lang, variant.get_message_key())),
        }
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum AlertVariant {
    LoginSuccess,
    LogoutSuccess,
    RegisterSuccess,
    ResetPasswordConfirmSuccess,
    ResetPasswordConfirmCodeNotEqual,
}

impl AlertVariant {
    pub fn get_message_key(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "auth.alert.login.success",
            Self::LogoutSuccess => "auth.alert.logout.success",
            Self::RegisterSuccess => "auth.alert.register.success",
            Self::ResetPasswordConfirmSuccess => "auth.alert.reset_password_confirm.success",
            Self::ResetPasswordConfirmCodeNotEqual => {
                "auth.alert.reset_password_confirm.code_not_equal"
            }
        }
    }
}

model_redis_impl!(Alert);
