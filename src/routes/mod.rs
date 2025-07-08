pub mod api;
pub mod web;
pub mod static_files;

pub fn register(cfg: &mut actix_web::web::ServiceConfig) {
    api::register(cfg);
    static_files::register(cfg);
    web::register(cfg);
}