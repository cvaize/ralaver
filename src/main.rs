mod app;
mod config;
mod connections;
mod helpers;
mod migrations;
mod routes;
mod schema;
mod services;

pub use crate::app::connections::mysql as mysql_connection;
pub use crate::app::connections::redis as redis_connection;
pub use crate::app::models::*;
pub use crate::app::services::*;
pub use crate::config::Config;
pub use crate::connections::Connections;
pub use crate::mysql_connection::MysqlPool;
pub use crate::services::Services;
use actix_session::SessionMiddleware;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use app::middlewares::error_redirect::ErrorRedirect;

async fn preparation() -> (Connections, Services<'static>) {
    dotenv::dotenv().ok();
    let base_services = services::base(Config::new());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let all_connections: Connections = connections::all(&base_services).await;

    migrations::migrate(&all_connections);

    let advanced_services = services::advanced(&all_connections, &base_services);

    let all_services: Services = services::join_to_all(base_services, advanced_services);

    (all_connections, all_services)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (all_connections, all_services) = preparation().await;

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                all_connections.session_redis_store.clone(),
                all_connections.session_redis_secret.clone(),
            ))
            .wrap(middleware::Logger::default())
            .app_data(all_services.config.clone())
            .app_data(all_services.log.clone())
            .app_data(all_services.key_value.clone())
            .app_data(all_services.translator.clone())
            .app_data(all_services.template.clone())
            .app_data(all_services.session.clone())
            .app_data(all_services.alert.clone())
            .app_data(all_services.hash.clone())
            .app_data(all_services.auth.clone())
            .app_data(all_services.locale.clone())
            .app_data(all_services.app.clone())
            .app_data(all_services.mail.clone())
            .app_data(all_services.rand.clone())
            .configure(routes::register)
            .wrap(ErrorRedirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
