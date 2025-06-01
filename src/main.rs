#![feature(test)]
extern crate test;

pub mod app;
pub mod config;
pub mod connections;
pub mod helpers;
pub mod libs;
pub mod migrations;
pub mod routes;
pub mod services;

use crate::app::controllers::web::errors::default_error_handler;
use crate::services::BaseServices;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::App;
use actix_web::HttpServer;
pub use app::connections::mysql as mysql_connection;
pub use app::connections::redis as redis_connection;
pub use app::controllers::web::WebHttpRequest;
pub use app::controllers::web::WebHttpResponse;
pub use app::dto::*;
pub use app::services::*;
pub use app::repositories::*;
pub use config::Config;
pub use connections::Connections;
pub use mysql_connection::MysqlPool;
pub use mysql_connection::MysqlPooledConnection;
pub use services::Services;

fn preparation() -> (Connections, Services) {
    dotenv::dotenv().ok();
    let base_services: BaseServices = services::base(Config::new());
    let _ = env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"));

    let all_connections: Connections = connections::all(&base_services);

    let advanced_services = services::advanced(&all_connections, &base_services);

    let all_services: Services = services::join_to_all(base_services, advanced_services);

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
            .wrap(Logger::default())
            .configure(routes::register)
            .wrap(ErrorHandlers::new().default_handler(default_error_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
