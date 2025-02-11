use actix_web::{web};
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/js/app.js").route(web::get().to(controllers::js::app)));
}