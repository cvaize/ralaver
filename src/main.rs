mod app;
mod config;
mod helpers;
mod routes;
mod schema;

pub use crate::app::connections::redis as redis_connection;
pub use crate::app::connections::mysql as mysql_connection;
pub use crate::app::models::*;
pub use crate::app::services::*;
pub use crate::config::Config;
pub use crate::mysql_connection::MysqlPool;
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
    // LogService
    let log_service = LogService::new(config.clone());

    // Db
    let mysql_pool: MysqlPool = mysql_connection::get_connection_pool(&config, &log_service)
        .expect("Failed to create connection MysqlPool.");
    let mysql_pool_data: Data<MysqlPool> = Data::new(mysql_pool);
    let mut connection = mysql_pool_data.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
    // Redis
    let session_redis_secret = redis_connection::get_session_secret(&config);
    let session_redis_store = redis_connection::get_session_store(&config, &log_service)
        .await
        .expect("Failed to create session redis store.");

    let redis_pool = redis_connection::get_connection_pool(&config, &log_service)
        .expect("Failed to create redis Pool.");

    // Services
    let log = Data::new(log_service);
    let key_value = Data::new(KeyValueService::new(redis_pool, log.clone()));
    let translator = Data::new(TranslatorService::new_from_files(
        config.clone(),
        log.clone(),
    )?);
    let template = Data::new(TemplateService::new_from_files(
        config.clone(),
        log.clone(),
    )?);
    let session = Data::new(SessionService::new(config.clone(), log.clone()));
    let alert = Data::new(AlertService::new(
        config.clone(),
        session.clone(),
        log.clone(),
    ));
    let hash = Data::new(HashService::new(Argon2::default(), log.clone()));
    let auth = Data::new(AuthService::new(
        config.clone(),
        mysql_pool_data.clone(),
        hash.clone(),
        key_value.clone(),
        log.clone(),
        session.clone(),
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
                session_redis_store.clone(),
                session_redis_secret.clone(),
            ))
            .wrap(middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(log.clone())
            .app_data(translator.clone())
            .app_data(template.clone())
            .app_data(alert.clone())
            .app_data(session.clone())
            .app_data(auth.clone())
            .app_data(app.clone())
            .app_data(locale.clone())
            .app_data(hash.clone())
            .app_data(mail.clone())
            .app_data(key_value.clone())
            .app_data(mysql_pool_data.clone())
            .configure(routes::register)
            .wrap(ErrorRedirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
