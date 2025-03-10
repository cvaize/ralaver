mod app;
mod routes;
mod db_connection;
mod schema;
mod config;

use dotenv::dotenv;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use actix_web::web;
use actix_web::middleware;
use actix_web::App;
use actix_web::HttpServer;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::cookie::Key;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db_pool = web::Data::new(db_connection::get_connection_pool());
    let mut connection = db_pool.get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);

    let redis_url: String = env::var("REDIS_URL").unwrap_or("redis://redis:6379".to_string());
    let redis_secret: String = env::var("REDIS_SECRET").unwrap_or("redis_secret".to_string());
    let redis_secret = Key::from(redis_secret.as_bytes());
    let redis_store = RedisSessionStore::new(redis_url)
        .await
        .unwrap();
    // let tt = core::template::new();

    log::info!("Starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::new(
                    redis_store.clone(),
                    redis_secret.clone(),
                )
            )
            .wrap(middleware::Logger::default())
            .configure(app::providers::config::register)
            .app_data(db_pool.clone())
            .configure(app::providers::routes::register)
            .configure(app::providers::template::register)
            // .app_data(web::Data::new(tt))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
