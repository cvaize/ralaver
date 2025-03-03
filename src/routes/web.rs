use actix_web::{web};
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(controllers::web::home::index)));
    cfg.service(web::resource("/users").route(web::get().to(controllers::web::users::index)));
    cfg.service(web::resource("/login").route(web::get().to(controllers::web::auth::login)));
    cfg.service(web::resource("/register").route(web::get().to(controllers::web::auth::register)));
    cfg.service(web::resource("/forgot-password").route(web::get().to(controllers::web::auth::forgot_password)));
    cfg.service(web::resource("/forgot-password-confirm").route(web::get().to(controllers::web::auth::forgot_password_confirm)));

    // NotFound route
    cfg.service(web::scope("").wrap(controllers::web::errors::error_handlers()));
}