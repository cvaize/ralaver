use actix_web::{HttpResponse, Responder};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Home page")
}

pub async fn test() -> impl Responder {
    HttpResponse::Ok().body("Test page")
}