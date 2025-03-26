use crate::config;
use crate::config::app::App;
use actix_web::web;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    pub app: App,
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.app_data(web::Data::new(Config {
        app: config::app::build(),
    }));
}
