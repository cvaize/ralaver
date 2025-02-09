use actix_web::{web, Resource};

pub fn new() -> Resource {
    web::resource("/")
        .route(web::get().to(crate::app::controllers::web::home::index))
}