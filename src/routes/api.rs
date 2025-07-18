use actix_web::web;

use crate::app::controllers::api::v1;
use crate::app::middlewares::web_auth::WebAuthMiddleware;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v1")
            .wrap(WebAuthMiddleware)
            .route(web::get().to(v1::index::index)),
    );
}
