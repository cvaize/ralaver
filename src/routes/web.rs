use crate::app::controllers;
use crate::app::middlewares::web::WebMiddleware;
use actix_web::web;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .wrap(WebMiddleware::build(true))
            .route(web::get().to(controllers::web::home::index)),
    );
    cfg.service(
        web::resource("/locale/switch").route(web::post().to(controllers::web::locale::switch)),
    );
    cfg.service(
        web::resource("/login")
            .wrap(WebMiddleware::build(false))
            .route(web::get().to(controllers::web::auth::login::show))
            .route(web::post().to(controllers::web::auth::login::invoke)),
    );
    cfg.service(
        web::resource("/logout")
            .wrap(WebMiddleware::build(false))
            .route(web::post().to(controllers::web::auth::logout::invoke)),
    );
    cfg.service(
        web::resource("/register")
            .wrap(WebMiddleware::build(false))
            .route(web::get().to(controllers::web::auth::register::show))
            .route(web::post().to(controllers::web::auth::register::invoke)),
    );
    cfg.service(
        web::resource("/reset-password")
            .wrap(WebMiddleware::build(false))
            .route(web::get().to(controllers::web::auth::reset_password::show))
            .route(web::post().to(controllers::web::auth::reset_password::invoke)),
    );
    cfg.service(
        web::resource("/reset-password-confirm")
            .wrap(WebMiddleware::build(false))
            .route(web::get().to(controllers::web::auth::reset_password_confirm::show))
            .route(web::post().to(controllers::web::auth::reset_password_confirm::invoke)),
    );
    cfg.service(
        web::resource("/profile")
            .wrap(WebMiddleware::build(true))
            .route(web::get().to(controllers::web::profile::index)),
    );
    cfg.service(
        web::resource("/users")
            .wrap(WebMiddleware::build(true))
            .route(web::post().to(controllers::web::users::index)),
    );

    // NotFound route
    cfg.service(
        web::scope("")
            .wrap(WebMiddleware::build(false))
            .wrap(controllers::web::errors::error_handlers()),
    );
}
