#![feature(test)]
extern crate core;
extern crate test;

pub mod app;
pub mod config;
pub mod errors;
pub mod helpers;
pub mod libs;
pub mod migrations;
pub mod routes;

use crate::app::connections::smtp::{get_smtp_transport, LettreSmtpTransport};
use crate::app::controllers::web::errors::default_error_handler;
use crate::redis_connection::RedisPool;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
pub use app::adapters::*;
pub use app::connections::mysql as mysql_connection;
pub use app::connections::mysql::get_connection_pool as get_mysql_connection_pool;
pub use app::connections::redis as redis_connection;
pub use app::connections::redis::get_connection_pool as get_redis_connection_pool;
pub use app::controllers::web::WebHttpRequest;
pub use app::controllers::web::WebHttpResponse;
pub use app::dto::*;
pub use app::policies::*;
pub use app::repositories::*;
pub use app::services::*;
pub use config::Config;
pub use errors::AppError;
pub use mysql_connection::MysqlPool;
pub use mysql_connection::MysqlPooledConnection;
use std::path::MAIN_SEPARATOR_STR;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _ = env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Connections
    let smtp: Data<LettreSmtpTransport> =
        Data::new(get_smtp_transport(&config.mail.smtp).unwrap());
    let mysql: Data<MysqlPool> =
        Data::new(get_mysql_connection_pool(&config.db.mysql).unwrap());
    let redis: Data<RedisPool> =
        Data::new(get_redis_connection_pool(&config.db.redis).unwrap());

    let kv_repository =
        Data::new(KVRepository::new(&config.db.kv.storage).expect("Fail init KVRepository::new"));

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        // Repositories
        let redis_repository = Data::new(RedisRepository::new(redis.clone()));
        let role_mysql_repository = Data::new(RoleMysqlRepository::new(mysql.clone()));
        let user_mysql_repository = Data::new(UserMysqlRepository::new(mysql.clone()));
        let disk_local_repository = Data::new(DiskLocalRepository::new(
            &config.filesystem.disks.local.root,
            &config.filesystem.disks.local.public_root,
            MAIN_SEPARATOR_STR,
        ));
        let disk_external_repository = Data::new(DiskExternalRepository::new());
        let file_mysql_repository = Data::new(FileMysqlRepository::new(mysql.clone()));
        let user_file_mysql_repository = Data::new(UserFileMysqlRepository::new(mysql.clone()));

        // Services
        let key_value_service = Data::new(KeyValueService::new(redis.clone()));
        let translator_service = Data::new(
            TranslatorService::new_from_files(config.clone())
                .expect("Fail init TranslatorService::new_from_files"),
        );
        let template_service = Data::new(
            TemplateService::new_from_files(config.clone())
                .expect("Fail init TemplateService::new_from_files"),
        );
        let rand_service = Data::new(RandomService::new());

        let hash_service = Data::new(HashService::new(config.clone()));
        let user_service = Data::new(UserService::new(
            hash_service.clone(),
            user_mysql_repository.clone(),
        ));

        let crypt_service = Data::new(CryptService::new(
            config.clone(),
            rand_service.clone(),
            hash_service.clone(),
        ));
        let auth_service = Data::new(AuthService::new(
            key_value_service.clone(),
            hash_service.clone(),
            user_service.clone(),
        ));
        let locale_service = Data::new(LocaleService::new(config.clone()));
        let app_service = Data::new(AppService::new(config.clone(), locale_service.clone()));
        let mail_service = Data::new(MailService::new(config.clone(), smtp.clone()));
        let rate_limit_service = Data::new(RateLimitService::new(key_value_service.clone()));
        let web_auth_service = Data::new(WebAuthService::new(
            config.clone(),
            crypt_service.clone(),
            rand_service.clone(),
            key_value_service.clone(),
            hash_service.clone(),
            user_service.clone(),
        ));

        let role_service = Data::new(RoleService::new(role_mysql_repository.clone()));

        let user_file_service = Data::new(UserFileService::new(
            config.clone(),
            user_file_mysql_repository.clone(),
            disk_local_repository.clone(),
        ));
        let file_service = Data::new(FileService::new(
            config.clone(),
            file_mysql_repository.clone(),
            user_file_service.clone(),
            disk_local_repository.clone(),
            disk_external_repository.clone(),
            rand_service.clone(),
            hash_service.clone(),
        ));
        let config: Data<Config> = Data::new(config.clone());
        App::new()
            .app_data(config)
            .app_data(smtp.clone())
            .app_data(mysql.clone())
            .app_data(redis.clone())
            .app_data(redis_repository)
            .app_data(role_mysql_repository)
            .app_data(user_mysql_repository)
            .app_data(disk_local_repository)
            .app_data(disk_external_repository)
            .app_data(file_mysql_repository)
            .app_data(user_file_mysql_repository)
            .app_data(kv_repository.clone())
            .app_data(key_value_service)
            .app_data(translator_service)
            .app_data(template_service)
            .app_data(hash_service)
            .app_data(auth_service)
            .app_data(web_auth_service)
            .app_data(locale_service)
            .app_data(app_service)
            .app_data(mail_service)
            .app_data(rand_service)
            .app_data(user_service)
            .app_data(crypt_service)
            .app_data(rate_limit_service)
            .app_data(role_service)
            .app_data(file_service)
            .app_data(user_file_service)
            .wrap(Logger::default())
            .configure(routes::register)
            .wrap(ErrorHandlers::new().default_handler(default_error_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
