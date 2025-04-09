mod app;
mod config;
mod db_connection;
mod helpers;
mod routes;
mod schema;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use argon2::Argon2;
use app::middlewares::error_redirect::ErrorRedirect;
pub use crate::db_connection::DbPool;
pub use crate::app::models::{*};
pub use crate::app::services::{*};
pub use crate::config::Config;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let config = Data::new(Config::new_from_env());
    // Db
    let db_pool: Data<DbPool> = Data::new(db_connection::get_connection_pool(config.get_ref()));
    let mut connection = db_pool.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
    // Redis
    let redis_url: String = env::var("REDIS_URL").unwrap_or("redis://redis:6379".to_string());
    let redis_secret: String = env::var("REDIS_SECRET").unwrap_or("redis_secret".to_string());
    let redis_secret = Key::from(redis_secret.as_bytes());
    let redis_store = RedisSessionStore::new(redis_url).await.unwrap();
    // Services
    let translator = Data::new(TranslatorService::new_from_files(config.clone())?);
    let template = Data::new(TemplateService::new_from_files(config.clone())?);
    let session = Data::new(SessionService::new(config.clone()));
    let alert = Data::new(AlertService::new(config.clone(), session.clone()));
    let hash = Data::new(HashService::new(Argon2::default()));
    let auth = Data::new(AuthService::new(config.clone(), db_pool.clone(), hash.clone()));
    let locale = Data::new(LocaleService::new(config.clone(), session.clone()));
    let app = Data::new(AppService::new(config.clone(), locale.clone(), alert.clone()));
    let mail = Data::new(MailService::new(config.clone(), None));

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                redis_secret.clone(),
            ))
            .wrap(middleware::Logger::default())
            .app_data(config.clone())
            .app_data(translator.clone())
            .app_data(db_pool.clone())
            .app_data(template.clone())
            .app_data(alert.clone())
            .app_data(session.clone())
            .app_data(auth.clone())
            .app_data(app.clone())
            .app_data(locale.clone())
            .app_data(hash.clone())
            .app_data(mail.clone())
            .configure(routes::register)
            .wrap(ErrorRedirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
