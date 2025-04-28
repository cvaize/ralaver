use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
    pub translator: TranslatorConfig,
    pub template: TemplateConfig,
    pub mail: MailConfig,
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub mysql: MysqlDbConfig,
    pub redis: RedisDbConfig,
}

#[derive(Debug, Clone)]
pub struct MysqlDbConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct RedisDbConfig {
    pub url: String,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub key: String,
    pub url: String,
    pub locale: String,
    pub fallback_locale: String,
    pub dark_mode_cookie_key: String,
    pub locale_cookie_key: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    // in seconds
    pub csrf_expires: u64,
    // in seconds
    pub token_expires: u64,
    // in seconds
    pub old_token_expires: u64,
    pub token_length: usize,
    pub token_cookie_key: String,
    pub token_cookie_secure: bool,
    pub token_cookie_http_only: bool,
    pub token_cookie_path: String,
    pub token_cookie_domain: String,
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
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self {
            app: AppConfig {
                key: env::var("APP_KEY")
                    .unwrap()
                    .trim()
                    .to_string(),
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
            },
            db: DbConfig {
                mysql: MysqlDbConfig {
                    url: env::var("MYSQL_URL")
                        .unwrap_or("mysql://test_user:test_password@mysql/test_db_name".to_string())
                        .trim()
                        .to_string(),
                },
                redis: RedisDbConfig {
                    url: env::var("REDIS_URL")
                        .unwrap_or("redis://redis:6379/app_db".to_string())
                        .trim()
                        .to_string(),
                    secret: env::var("REDIS_SECRET")
                        .unwrap_or("redis_secret".to_string())
                        .trim()
                        .to_string(),
                },
            },
            auth: AuthConfig {
                csrf_expires: env::var("AUTH_CSRF_EXPIRES")
                    // Default: 24 hours equal 86400 seconds
                    .unwrap_or("86400".to_string())
                    .trim()
                    .parse::<u64>().unwrap_or(86400),
                token_expires: env::var("AUTH_TOKEN_EXPIRES")
                    // Default: 30 days equal 2592000 seconds
                    .unwrap_or("2592000".to_string())
                    .trim()
                    .parse::<u64>().unwrap_or(2592000),
                old_token_expires: env::var("AUTH_OLD_TOKEN_EXPIRES")
                    // Default: 10 minutes equal 600 seconds
                    .unwrap_or("600".to_string())
                    .trim()
                    .parse::<u64>().unwrap_or(600),
                token_length: env::var("AUTH_TOKEN_LENGTH")
                    .unwrap_or("64".to_string())
                    .trim()
                    .parse::<usize>().unwrap_or(64),
                token_cookie_key: env::var("AUTH_TOKEN_COOKIE_KEY")
                    .unwrap_or("access_token".to_string())
                    .trim()
                    .to_string(),
                token_cookie_http_only: env::var("AUTH_TOKEN_COOKIE_HTTP_ONLY")
                    .unwrap_or("1".to_string())
                    .trim()
                    .parse::<bool>().unwrap_or(true),
                token_cookie_path: env::var("AUTH_TOKEN_COOKIE_PATH")
                    .unwrap_or("/".to_string())
                    .trim()
                    .to_string(),
                token_cookie_secure: env::var("AUTH_TOKEN_COOKIE_SECURE")
                    .unwrap_or("0".to_string())
                    .trim()
                    .parse::<bool>().unwrap_or(false),
                token_cookie_domain: env::var("AUTH_TOKEN_COOKIE_DOMAIN")
                    .unwrap_or("".to_string())
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
