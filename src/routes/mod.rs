pub mod api;
pub mod css;
pub mod js;
pub mod svg;
pub mod web;

pub fn register(cfg: &mut actix_web::web::ServiceConfig) {
    api::register(cfg);
    css::register(cfg);
    js::register(cfg);
    svg::register(cfg);
    web::register(cfg);
}