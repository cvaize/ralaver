use dotenv::dotenv;

use actix_web::web;
use actix_web::middleware;
use actix_web::App;
use actix_web::HttpServer;

mod app;
mod core;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://0.0.0.0:8080");

    HttpServer::new(|| {
        let tt = core::template::new();

        App::new()
            .wrap(middleware::Logger::default())
            .configure(app::providers::routes::register)
            .app_data(web::Data::new(tt))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
