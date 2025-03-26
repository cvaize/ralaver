use std::env;
use dotenv::dotenv;

#[allow(dead_code)]
#[derive(Debug)]
pub struct App {
    pub locale: String,
    pub fallback_locale: String,
}

pub fn build() -> App {
    dotenv().ok();

    App {
        locale: env::var("APP_LOCALE").unwrap_or("en".to_string()),
        fallback_locale: env::var("APP_FALLBACK_LOCALE").unwrap_or("en".to_string()),
    }
}
