use actix_web::{web};
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(controllers::web::home::index)));
    cfg.service(web::resource("/users").route(web::get().to(controllers::web::users::index)));

    // NotFound route
    cfg.service(web::scope("").wrap(controllers::web::errors::error_handlers()));
}