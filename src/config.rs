use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
    pub alerts: AlertsConfig,
    pub translator: TranslatorConfig,
    pub template: TemplateConfig,
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub mysql: MysqlDbConfig,
}

#[derive(Debug, Clone)]
pub struct MysqlDbConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub locale: String,
    pub fallback_locale: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub user_id_session_key: String,
}

#[derive(Debug, Clone)]
pub struct AlertsConfig {
    pub session_key: String,
}

#[derive(Debug, Clone)]
pub struct TranslatorConfig {
    pub translates_folder: String,
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub handlebars: HandlebarsTemplateConfig,
}

#[derive(Debug, Clone)]
pub struct HandlebarsTemplateConfig {
    pub folder: String,
}

impl Config {
    pub fn new_from_env() -> Self {
        Self {
            app: AppConfig {
                locale: env::var("APP_LOCALE").unwrap_or("en".to_string()),
                fallback_locale: env::var("APP_FALLBACK_LOCALE").unwrap_or("en".to_string()),
            },
            db: DbConfig {
                mysql: MysqlDbConfig {
                    url: env::var("MYSQL_URL").unwrap_or("mysql://test_user:test_password@mysql/test_db_name".to_string()),
                }
            },
            auth: AuthConfig {
                user_id_session_key: env::var("AUTH_USER_ID_SESSION_KEY").unwrap_or("app.auth.user.id".to_string()),
            },
            alerts: AlertsConfig {
                session_key: env::var("ALERTS_SESSION_KEY").unwrap_or("app.alerts".to_string()),
            },
            translator: TranslatorConfig {
                translates_folder: env::var("TRANSLATOR_TRANSLATES_FOLDER").unwrap_or("resources/lang".to_string()),
            },
            template: TemplateConfig {
                handlebars: HandlebarsTemplateConfig {
                    folder: env::var("TEMPLATE_HANDLEBARS_FOLDER").unwrap_or("resources/handlebars".to_string()),
                }
            }
        }
    }
}