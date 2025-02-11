use actix_web::{web};
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/css/app.css").route(web::get().to(controllers::css::app)));
}