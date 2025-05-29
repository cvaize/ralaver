use std::collections::HashMap;
use crate::{model_redis_impl, TranslatorService};
use serde_bare;
use serde_derive::{Deserialize, Serialize};

pub static ALERTS_KEY: &str = "alerts";

#[derive(Serialize, Deserialize, Debug)]
pub struct Alert {
    pub style: String,
    pub content: String,
}

macro_rules! one_variables {
    ($name:expr, $value:expr) => {
        {
            let mut vars: HashMap<&str, &str> = HashMap::new();
            vars.insert($name, $value);
            vars
        }
    }
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
    pub fn from_variant(t_s: &TranslatorService, lang: &str, variant: &AlertVariant) -> Self {
        match variant {
            AlertVariant::LoginSuccess => {
                Self::success(t_s.translate(&lang, "alert.login.success"))
            }
            AlertVariant::LogoutSuccess => {
                Self::success(t_s.translate(&lang, "alert.logout.success"))
            }
            AlertVariant::RegisterSuccess => {
                Self::success(t_s.translate(&lang, "alert.register.success"))
            }
            AlertVariant::ResetPasswordConfirmSuccess => {
                Self::success(t_s.translate(&lang, "alert.reset_password_confirm.success"))
            }
            AlertVariant::ResetPasswordConfirmCodeNotEqual => {
                Self::error(t_s.translate(&lang, "alert.reset_password_confirm.code_not_equal"))
            }
            AlertVariant::UsersCreateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(t_s.variables(&lang, "alert.users.create.success", &vars))
            }
            AlertVariant::UsersUpdateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(t_s.variables(&lang, "alert.users.update.success", &vars))
            }
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AlertVariant {
    LoginSuccess,
    LogoutSuccess,
    RegisterSuccess,
    ResetPasswordConfirmSuccess,
    ResetPasswordConfirmCodeNotEqual,
    UsersCreateSuccess(String),
    UsersUpdateSuccess(String)
}

impl AlertVariant {
    pub fn to_string(&self) -> String {
        // {AlertVariant}::{var1}::{var2}::{var3}
        match self {
            Self::LoginSuccess => "login_success".to_string(),
            Self::LogoutSuccess => "logout_success".to_string(),
            Self::RegisterSuccess => "register_success".to_string(),
            Self::ResetPasswordConfirmSuccess => "reset_password_confirm_success".to_string(),
            Self::ResetPasswordConfirmCodeNotEqual => "reset_password_confirm_code_not_equal".to_string(),
            Self::UsersCreateSuccess(name) => {
                let mut str = "users_create_success::".to_string();
                str.push_str(name);
                str
            }
            Self::UsersUpdateSuccess(name) => {
                let mut str = "users_update_success::".to_string();
                str.push_str(name);
                str
            }
        }
    }

    pub fn from_string(string: &str) -> Result<Self, ParseAlertVariantError> {
        // {AlertVariant}::{var1}::{var2}::{var3}
        let string: Vec<&str> = string.split("::").collect();
        let id = string.get(0).ok_or(ParseAlertVariantError)?;

        match *id {
            "login_success" => Ok(Self::LoginSuccess),
            "logout_success" => Ok(Self::LogoutSuccess),
            "register_success" => Ok(Self::RegisterSuccess),
            "reset_password_confirm_success" => Ok(Self::ResetPasswordConfirmSuccess),
            "reset_password_confirm_code_not_equal" => Ok(Self::ResetPasswordConfirmCodeNotEqual),
            "users_create_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::UsersCreateSuccess(p.to_string()))
            },
            "users_update_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::UsersUpdateSuccess(p.to_string()))
            },
            _ => Err(ParseAlertVariantError)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseAlertVariantError;

model_redis_impl!(Alert);

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_alert_variant_to_string() {
        let v: String = AlertVariant::UsersCreateSuccess("Test,test,test.:test.".to_string()).to_string();
        assert_eq!("users_create_success::Test,test,test.:test.".to_string(), v);
    }

    #[test]
    fn test_alert_variant_from_string() {
        let v = "users_create_success::Test,test,test.:test.".to_string();
        let a = AlertVariant::UsersCreateSuccess("Test,test,test.:test.".to_string());
        assert_eq!(a, AlertVariant::from_string(&v).unwrap());
    }

    #[bench]
    fn bench_alert_variant_to_string(b: &mut Bencher) {
        // 120.32 ns/iter (+/- 2.26)
        b.iter(|| {
            let _ = AlertVariant::UsersCreateSuccess("Test,test,test.:test.".to_string()).to_string();
        });
    }

    #[bench]
    fn bench_alert_variant_from_string(b: &mut Bencher) {
        // 155.49 ns/iter (+/- 2.36)
        b.iter(|| {
            let _ = AlertVariant::from_string("users_create_success::Test,test,test.:test.").unwrap();
        });
    }
}