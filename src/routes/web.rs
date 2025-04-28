use crate::app::controllers;
use actix_web::web;
use crate::app::middlewares::web_auth::WebAuthMiddleware;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .wrap(WebAuthMiddleware)
            .route(web::get().to(controllers::web::home::index)),
    );
    cfg.service(web::resource("/locale/switch").route(web::post().to(controllers::web::locale::switch)));
    cfg.service(
        web::resource("/login")
            .route(web::get().to(controllers::web::auth::login::show))
            .route(web::post().to(controllers::web::auth::login::invoke)),
    );
    cfg.service(
        web::resource("/logout")
            .route(web::post().to(controllers::web::auth::logout::invoke)),
    );
    cfg.service(
        web::resource("/register")
            .route(web::get().to(controllers::web::auth::register::show))
            .route(web::post().to(controllers::web::auth::register::invoke)),
    );
    cfg.service(
        web::resource("/reset-password")
            .route(web::get().to(controllers::web::auth::reset_password::show))
            .route(web::post().to(controllers::web::auth::reset_password::invoke)),
    );
    cfg.service(
        web::resource("/reset-password-confirm")
            .route(web::get().to(controllers::web::auth::reset_password_confirm::show))
            .route(web::post().to(controllers::web::auth::reset_password_confirm::invoke)),
    );
    cfg.service(
        web::resource("/profile")
            .wrap(WebAuthMiddleware)
            .route(web::get().to(controllers::web::profile::index)),
    );
    cfg.service(
        web::resource("/users")
            .wrap(WebAuthMiddleware)
            .route(web::post().to(controllers::web::users::index)),
    );

    // NotFound route
    cfg.service(web::scope("").wrap(controllers::web::errors::error_handlers()));
}
