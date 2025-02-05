use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Home page")
}

#[get("/test")]
pub async fn test() -> impl Responder {
    HttpResponse::Ok().body("Test page")
}