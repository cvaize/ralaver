use crate::app::controllers::static_files;
use crate::app::middlewares::web_auth::WebAuthMiddleware;
use actix_web::web;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/css/app.css").route(web::get().to(static_files::css::app)));
    cfg.service(web::resource("/js/app.js").route(web::get().to(static_files::js::app)));
    cfg.service(web::resource("/svg/logo.svg").route(web::get().to(static_files::svg::logo)));
    cfg.service(
        web::resource("/storage/files/{filename}")
            .route(web::get().to(static_files::storage::public)),
    );
    cfg.service(
        web::resource("/storage/private-files/{filename}")
            .wrap(WebAuthMiddleware)
            .route(web::get().to(static_files::storage::private)),
    );
}
