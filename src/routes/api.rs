use actix_web::{web};

use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/v1").route(web::get().to(controllers::api::v1::index::index)));
}