use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
    pub alerts: AlertsConfig,
    pub translator: TranslatorConfig,
    pub template: TemplateConfig,
    pub mail: MailConfig,
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
    pub url: String,
    pub locale: String,
    pub fallback_locale: String,
    pub dark_mode_cookie_key: String,
    pub locale_cookie_key: String,
    pub locale_session_key: String,
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

#[derive(Debug, Clone)]
pub struct MailConfig {
    pub smtp: MailSmtpConfig,
}

#[derive(Debug, Clone)]
pub struct MailSmtpConfig {
    pub host: String,
    pub port: String,
    pub encryption: String,
    pub from_name: String,
    pub from_address: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn new_from_env() -> Self {
        Self {
            app: AppConfig {
                url: env::var("APP_URL")
                    .unwrap_or("http://localhost".to_string())
                    .trim()
                    .to_string(),
                locale: env::var("APP_LOCALE")
                    .unwrap_or("en".to_string())
                    .trim()
                    .to_string(),
                fallback_locale: env::var("APP_FALLBACK_LOCALE")
                    .unwrap_or("en".to_string())
                    .trim()
                    .to_string(),
                dark_mode_cookie_key: env::var("APP_DARK_MODE_COOKIE_KEY")
                    .unwrap_or("dark_mode".to_string())
                    .trim()
                    .to_string(),
                locale_cookie_key: env::var("APP_LOCALE_COOKIE_KEY")
                    .unwrap_or("locale".to_string())
                    .trim()
                    .to_string(),
                locale_session_key: env::var("APP_LOCALE_SESSION_KEY")
                    .unwrap_or("app.user.locale".to_string())
                    .trim()
                    .to_string(),
            },
            db: DbConfig {
                mysql: MysqlDbConfig {
                    url: env::var("MYSQL_URL")
                        .unwrap_or("mysql://test_user:test_password@mysql/test_db_name".to_string())
                        .trim()
                        .to_string(),
                },
            },
            auth: AuthConfig {
                user_id_session_key: env::var("AUTH_USER_ID_SESSION_KEY")
                    .unwrap_or("app.auth.user.id".to_string())
                    .trim()
                    .to_string(),
            },
            alerts: AlertsConfig {
                session_key: env::var("ALERTS_SESSION_KEY")
                    .unwrap_or("app.alerts".to_string())
                    .trim()
                    .to_string(),
            },
            translator: TranslatorConfig {
                translates_folder: env::var("TRANSLATOR_TRANSLATES_FOLDER")
                    .unwrap_or("resources/lang".to_string())
                    .trim()
                    .to_string(),
            },
            template: TemplateConfig {
                handlebars: HandlebarsTemplateConfig {
                    folder: env::var("TEMPLATE_HANDLEBARS_FOLDER")
                        .unwrap_or("resources/handlebars".to_string())
                        .trim()
                        .to_string(),
                },
            },
            mail: MailConfig {
                // Add in the future transports: "sendmail", "mailgun", "ses", "ses-v2", "postmark", "resend", "log", "array", "failover", "roundrobin"
                smtp: MailSmtpConfig {
                    host: env::var("MAIL_HOST")
                        .unwrap_or("127.0.0.1".to_string())
                        .trim()
                        .to_string(),
                    port: env::var("MAIL_PORT")
                        .unwrap_or("8025".to_string())
                        .trim()
                        .to_string(),
                    // "", "tls"
                    encryption: env::var("MAIL_ENCRYPTION")
                        .unwrap_or("".to_string())
                        .trim()
                        .to_string(),
                    from_name: env::var("MAIL_FROM_NAME")
                        .unwrap_or("".to_string())
                        .trim()
                        .to_string(),
                    from_address: env::var("MAIL_FROM_ADDRESS")
                        .unwrap_or("".to_string())
                        .trim()
                        .to_string(),
                    username: env::var("MAIL_USERNAME")
                        .unwrap_or("".to_string())
                        .trim()
                        .to_string(),
                    password: env::var("MAIL_PASSWORD")
                        .unwrap_or("".to_string())
                        .trim()
                        .to_string(),
                },
            },
        }
    }
}
