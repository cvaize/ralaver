#![feature(test)]
extern crate core;
extern crate test;

pub mod app;
pub mod config;
pub mod connections;
pub mod errors;
pub mod helpers;
pub mod libs;
pub mod migrations;
pub mod routes;
pub mod services;

use crate::app::controllers::web::errors::default_error_handler;
use crate::services::Services;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
pub use app::connections::mysql as mysql_connection;
pub use app::connections::redis as redis_connection;
pub use app::controllers::web::WebHttpRequest;
pub use app::controllers::web::WebHttpResponse;
pub use app::dto::*;
pub use app::policies::*;
pub use app::repositories::*;
pub use app::services::*;
pub use config::Config;
pub use connections::Connections;
pub use errors::AppError;
pub use mysql_connection::MysqlPool;
pub use mysql_connection::MysqlPooledConnection;

fn preparation() -> (Connections, Services) {
    dotenv::dotenv().ok();
    let config = Data::new(Config::new());
    let _ = env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"));

    let all_connections: Connections = connections::all(config.get_ref());

    let all_services = services::build(&all_connections, config);

    (all_connections, all_services)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (_, all_services) = preparation();

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(all_services.config.clone())
            .app_data(all_services.key_value_service.clone())
            .app_data(all_services.translator_service.clone())
            .app_data(all_services.template_service.clone())
            .app_data(all_services.hash_service.clone())
            .app_data(all_services.auth_service.clone())
            .app_data(all_services.web_auth_service.clone())
            .app_data(all_services.locale_service.clone())
            .app_data(all_services.app_service.clone())
            .app_data(all_services.mail_service.clone())
            .app_data(all_services.rand_service.clone())
            .app_data(all_services.user_service.clone())
            .app_data(all_services.crypt_service.clone())
            .app_data(all_services.rate_limit_service.clone())
            .app_data(all_services.role_service.clone())
            .app_data(all_services.file_service.clone())
            .wrap(Logger::default())
            .configure(routes::register)
            .wrap(ErrorHandlers::new().default_handler(default_error_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
