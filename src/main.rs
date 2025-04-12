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
use actix_session::SessionMiddleware;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use app::middlewares::error_redirect::ErrorRedirect;
use argon2::Argon2;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use crate::app::connections::smtp::{get_smtp_transport, LettreSmtpTransport};
use crate::redis_connection::RedisPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let config = Data::new(Config::new_from_env());
    // LogService
    let log = Data::new(LogService::new());

    // Smtp
    let smtp: LettreSmtpTransport = get_smtp_transport(config.get_ref(), log.get_ref())
        .expect("Failed to create connection MysqlPool.");
    let smtp_data: Data<LettreSmtpTransport> = Data::new(smtp);
    // Db
    let mysql_pool: MysqlPool = mysql_connection::get_connection_pool(config.get_ref(), log.get_ref())
        .expect("Failed to create connection MysqlPool.");
    let mysql_pool_data: Data<MysqlPool> = Data::new(mysql_pool);
    let mut connection = mysql_pool_data.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
    // Redis
    let session_redis_secret = redis_connection::get_session_secret(config.get_ref());
    let session_redis_store = redis_connection::get_session_store(config.get_ref(), log.get_ref())
        .await
        .expect("Failed to create session redis store.");

    let redis_pool: RedisPool = redis_connection::get_connection_pool(config.get_ref(), log.get_ref())
        .expect("Failed to create redis Pool.");
    let redis_pool_data: Data<RedisPool> = Data::new(redis_pool);

    // Services (LogService above)
    let key_value = Data::new(KeyValueService::new(redis_pool_data.clone(), log.clone()));
    let translator = Data::new(TranslatorService::new_from_files(
        config.clone(),
        log.clone(),
    )?);
    let template = Data::new(TemplateService::new_from_files(
        config.clone(),
        log.clone(),
    )?);
    let session = Data::new(SessionService::new(log.clone()));
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
    let mail = Data::new(MailService::new(config.clone(), log.clone(), smtp_data));
    let rand = Data::new(RandomService::new());

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                session_redis_store.clone(),
                session_redis_secret.clone(),
            ))
            .wrap(middleware::Logger::default())
            .app_data(config.clone())
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
            .app_data(redis_pool_data.clone())
            .app_data(mysql_pool_data.clone())
            .app_data(rand.clone())
            .configure(routes::register)
            .wrap(ErrorRedirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
