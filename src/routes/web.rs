use crate::app::controllers;
use actix_web::web;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::get().to(controllers::web::home::index)),
    );
    cfg.service(web::resource("/users").route(web::get().to(controllers::web::users::index)));
    cfg.service(
        web::resource("/login")
            .route(web::get().to(controllers::web::auth::login::show))
            .route(web::post().to(controllers::web::auth::login::sign_in)),
    );
    cfg.service(
        web::resource("/logout")
            .route(web::get().to(controllers::web::auth::login::sign_out))
            .route(web::post().to(controllers::web::auth::login::sign_out)),
    );
    cfg.service(
        web::resource("/register")
            .route(web::get().to(controllers::web::auth::register::show))
            .route(web::post().to(controllers::web::auth::register::store)),
    );
    cfg.service(
        web::resource("/forgot-password")
            .route(web::get().to(controllers::web::auth::forgot_password::show)),
    );
    cfg.service(
        web::resource("/forgot-password-confirm")
            .route(web::get().to(controllers::web::auth::forgot_password_confirm::show)),
    );
    cfg.service(
        web::resource("/dark-mode")
            .route(web::post().to(controllers::web::session::dark_mode)),
    );

    // NotFound route
    cfg.service(web::scope("").wrap(controllers::web::errors::error_handlers()));
}
