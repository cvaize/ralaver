use dotenv::dotenv;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use actix_web::web;
use actix_web::middleware;
use actix_web::App;
use actix_web::HttpServer;

mod app;
mod core;
mod routes;
mod db_connection;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut app_addr: String = env::var("FORWARD_APP_ADDR").unwrap_or("0.0.0.0".to_string()).to_owned();
    let app_port: String = env::var("FORWARD_APP_PORT").unwrap_or("8080".to_string());

    app_addr.push_str(":");
    app_addr.push_str(&app_port);

    let db_pool = web::Data::new(db_connection::get_connection_pool());
    let mut connection = db_pool.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);

    log::info!("Starting HTTP server at http://{:}", app_addr);

    HttpServer::new(move || {
        let tt = core::template::new();

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(db_pool.clone())
            .configure(app::providers::routes::register)
            .app_data(web::Data::new(tt))
    })
    .bind(app_addr)?
    .run()
    .await
}
