use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;

mod app;
mod core;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        let tt = core::template::new();

        App::new()
            .app_data(web::Data::new(tt))
            .wrap(middleware::Logger::default())
            .service(routes::web::new())
            .service(web::scope("").wrap(app::controllers::web::errors::error_handlers()))
            // .service(web::scope("").wrap(error_handlers()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
