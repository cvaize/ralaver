use crate::{model_redis_impl, TranslatorService};
use serde_bare;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub static ALERTS_KEY: &str = "alerts";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alert {
    pub style: String,
    pub content: String,
}

macro_rules! one_variables {
    ($name:expr, $value:expr) => {{
        let mut vars: HashMap<&str, &str> = HashMap::new();
        vars.insert($name, $value);
        vars
    }};
}

macro_rules! two_variables {
    ($name1:expr, $value1:expr, $name2:expr, $value2:expr) => {{
        let mut vars: HashMap<&str, &str> = HashMap::new();
        vars.insert($name1, $value1);
        vars.insert($name2, $value2);
        vars
    }};
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
            AlertVariant::LoginSuccess => {
                Self::success(translator_service.translate(&lang, "alert.login.success"))
            }
            AlertVariant::LogoutSuccess => {
                Self::success(translator_service.translate(&lang, "alert.logout.success"))
            }
            AlertVariant::RegisterSuccess => {
                Self::success(translator_service.translate(&lang, "alert.register.success"))
            }
            AlertVariant::ResetPasswordConfirmSuccess => {
                Self::success(translator_service.translate(&lang, "alert.reset_password_confirm.success"))
            }
            AlertVariant::ResetPasswordConfirmCodeNotEqual => {
                Self::error(translator_service.translate(&lang, "alert.reset_password_confirm.code_not_equal"))
            }
            AlertVariant::UsersCreateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.users.create.success", &vars))
            }
            AlertVariant::UsersUpdateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.users.update.success", &vars))
            }
            AlertVariant::UsersDeleteSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.users.delete.success", &vars))
            }
            AlertVariant::UsersMassDeleteSuccess(ids) => {
                let vars = one_variables!("ids", ids);
                Self::success(translator_service.variables(&lang, "alert.users.mass_delete.success", &vars))
            }
            AlertVariant::ValidationRateLimitError(seconds, unit) => {
                let vars = two_variables!("seconds", seconds, "unit", unit);
                Self::success(translator_service.variables(&lang, "validation.rate_limit", &vars))
            }
            AlertVariant::RolesCreateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.roles.create.success", &vars))
            }
            AlertVariant::RolesUpdateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.roles.update.success", &vars))
            }
            AlertVariant::RolesDeleteSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.roles.delete.success", &vars))
            }
            AlertVariant::RolesMassDeleteSuccess(ids) => {
                let vars = one_variables!("ids", ids);
                Self::success(translator_service.variables(&lang, "alert.roles.mass_delete.success", &vars))
            }
            AlertVariant::FilesCreateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.files.create.success", &vars))
            }
            AlertVariant::FilesUpdateSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.files.update.success", &vars))
            }
            AlertVariant::FilesDeleteSuccess(name) => {
                let vars = one_variables!("name", name);
                Self::success(translator_service.variables(&lang, "alert.files.delete.success", &vars))
            }
            AlertVariant::FilesMassDeleteSuccess(ids) => {
                let vars = one_variables!("ids", ids);
                Self::success(translator_service.variables(&lang, "alert.files.mass_delete.success", &vars))
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
    UsersUpdateSuccess(String),
    UsersDeleteSuccess(String),
    UsersMassDeleteSuccess(String),
    ValidationRateLimitError(String, String),
    RolesCreateSuccess(String),
    RolesUpdateSuccess(String),
    RolesDeleteSuccess(String),
    RolesMassDeleteSuccess(String),
    FilesCreateSuccess(String),
    FilesUpdateSuccess(String),
    FilesDeleteSuccess(String),
    FilesMassDeleteSuccess(String),
}

impl AlertVariant {
    pub fn to_string(&self) -> String {
        // {AlertVariant}::{var1}::{var2}::{var3}
        match self {
            Self::LoginSuccess => "login_success".to_string(),
            Self::LogoutSuccess => "logout_success".to_string(),
            Self::RegisterSuccess => "register_success".to_string(),
            Self::ResetPasswordConfirmSuccess => "reset_password_confirm_success".to_string(),
            Self::ResetPasswordConfirmCodeNotEqual => {
                "reset_password_confirm_code_not_equal".to_string()
            }
            Self::UsersCreateSuccess(name) => {
                format!("users_create_success::{}", name)
            }
            Self::UsersUpdateSuccess(name) => {
                format!("users_update_success::{}", name)
            }
            Self::UsersDeleteSuccess(name) => {
                format!("users_delete_success::{}", name)
            }
            Self::UsersMassDeleteSuccess(ids) => {
                format!("users_mass_delete_success::{}", ids)
            }
            Self::ValidationRateLimitError(seconds, unit) => {
                format!("validation_rate_limit_error::{}::{}", seconds, unit)
            }
            Self::RolesCreateSuccess(name) => {
                format!("roles_create_success::{}", name)
            }
            Self::RolesUpdateSuccess(name) => {
                format!("roles_update_success::{}", name)
            }
            Self::RolesDeleteSuccess(name) => {
                format!("roles_delete_success::{}", name)
            }
            Self::RolesMassDeleteSuccess(ids) => {
                format!("roles_mass_delete_success::{}", ids)
            }
            Self::FilesCreateSuccess(name) => {
                format!("files_create_success::{}", name)
            }
            Self::FilesUpdateSuccess(name) => {
                format!("files_update_success::{}", name)
            }
            Self::FilesDeleteSuccess(name) => {
                format!("files_delete_success::{}", name)
            }
            Self::FilesMassDeleteSuccess(ids) => {
                format!("files_mass_delete_success::{}", ids)
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
            }
            "users_update_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::UsersUpdateSuccess(p.to_string()))
            }
            "users_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::UsersDeleteSuccess(p.to_string()))
            }
            "users_mass_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::UsersMassDeleteSuccess(p.to_string()))
            }
            "validation_rate_limit_error" => {
                let p1 = string.get(1).ok_or(ParseAlertVariantError)?;
                let p2 = string.get(2).ok_or(ParseAlertVariantError)?;
                Ok(Self::ValidationRateLimitError(p1.to_string(), p2.to_string()))
            }
            "roles_create_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::RolesCreateSuccess(p.to_string()))
            }
            "roles_update_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::RolesUpdateSuccess(p.to_string()))
            }
            "roles_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::RolesDeleteSuccess(p.to_string()))
            }
            "roles_mass_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::RolesMassDeleteSuccess(p.to_string()))
            }
            "files_create_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::FilesCreateSuccess(p.to_string()))
            }
            "files_update_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::FilesUpdateSuccess(p.to_string()))
            }
            "files_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::FilesDeleteSuccess(p.to_string()))
            }
            "files_mass_delete_success" => {
                let p = string.get(1).ok_or(ParseAlertVariantError)?;
                Ok(Self::FilesMassDeleteSuccess(p.to_string()))
            }
            _ => Err(ParseAlertVariantError),
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
        let v: String =
            AlertVariant::UsersCreateSuccess("Test,test,test.:test.".to_string()).to_string();
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
            let _ =
                AlertVariant::UsersCreateSuccess("Test,test,test.:test.".to_string()).to_string();
        });
    }

    #[bench]
    fn bench_alert_variant_from_string(b: &mut Bencher) {
        // 155.49 ns/iter (+/- 2.36)
        b.iter(|| {
            let _ =
                AlertVariant::from_string("users_create_success::Test,test,test.:test.").unwrap();
        });
    }
}
