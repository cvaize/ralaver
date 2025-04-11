mod app;
mod config;
mod db_connection;
mod helpers;
mod redis_connection;
mod routes;
mod schema;

pub use crate::app::models::*;
pub use crate::app::services::*;
pub use crate::config::Config;
pub use crate::db_connection::DbPool;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use app::middlewares::error_redirect::ErrorRedirect;
use argon2::Argon2;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // TODO: Измерить производительность использования config в Arc и без него: Data<Config> или Config.
    let config = Config::new_from_env();
    let config_data = Data::new(config.clone());
    // Db
    let db_pool: Data<DbPool> = Data::new(db_connection::get_connection_pool(&config));
    let mut connection = db_pool.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
    // Redis
    let redis_secret = Key::from(config.db.redis.secret.to_owned().as_bytes());
    let redis_store = RedisSessionStore::new(config.db.redis.url.to_owned())
        .await
        .unwrap();
    let redis_pool = redis_connection::get_connection_pool(&config);
    // Services
    let log = Data::new(LogService::new(config.clone()));
    let key_value = Data::new(KeyValueService::new(redis_pool, log.clone()));
    let translator = Data::new(TranslatorService::new_from_files(config.clone(), log.clone())?);
    let template = Data::new(TemplateService::new_from_files(config.clone(), log.clone())?);
    let session = Data::new(SessionService::new(config.clone(), log.clone()));
    let alert = Data::new(AlertService::new(
        config.clone(),
        session.clone(),
        log.clone(),
    ));
    let hash = Data::new(HashService::new(Argon2::default(), log.clone()));
    let auth = Data::new(AuthService::new(
        config.clone(),
        db_pool.clone(),
        hash.clone(),
        key_value.clone(),
        log.clone(),
        session.clone()
    ));
    let locale = Data::new(LocaleService::new(config.clone(), session.clone()));
    let app = Data::new(AppService::new(
        config.clone(),
        locale.clone(),
        alert.clone(),
    ));
    let mail = Data::new(MailService::new(config.clone(), log.clone(), None));

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                redis_secret.clone(),
            ))
            .wrap(middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(log.clone())
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
            .app_data(key_value.clone())
            .configure(routes::register)
            .wrap(ErrorRedirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
