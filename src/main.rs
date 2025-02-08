use actix_web::{web, App, HttpServer};

mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(app::controllers::web::home::index))
            .route("/test", web::get().to(app::controllers::web::home::test))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
