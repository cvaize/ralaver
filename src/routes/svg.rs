use actix_web::web;
use crate::app::controllers;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/svg/logo.svg").route(web::get().to(controllers::svg::logo)));
}