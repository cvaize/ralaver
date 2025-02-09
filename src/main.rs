use std::collections::HashMap;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    http::{header::ContentType, StatusCode},
    middleware,
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, App, HttpResponse, HttpServer, Result,
};
use tinytemplate::TinyTemplate;
use app::controllers::web::home as home_controller;

mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        let mut tt = TinyTemplate::new();
        tt.add_template("index.html", HTML_PAGE_HOME_INDEX).unwrap();
        tt.add_template("user.html", HTML_PAGE_HOME_USER).unwrap();
        tt.add_template("error.html", HTML_PAGE_ERROR_DEFAULT).unwrap();

        App::new()
            .app_data(web::Data::new(tt))
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(home_controller::index)))
            .service(web::scope("").wrap(error_handlers()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |err: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(err.to_string())
    };

    let tt = request
        .app_data::<web::Data<TinyTemplate<'_>>>()
        .map(|t| t.get_ref());
    match tt {
        Some(tt) => {
            let mut context = HashMap::new();
            context.insert("error", error.to_owned());
            context.insert("status_code", res.status().as_str().to_owned());
            let body = tt.render("error.html", &context);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}

static HTML_PAGE_ERROR_DEFAULT: &str = include_str!("../resources/view/pages/error/default.html");
static HTML_PAGE_HOME_INDEX: &str = include_str!("../resources/view/pages/home/index.html");
static HTML_PAGE_HOME_USER: &str = include_str!("../resources/view/pages/home/user.html");
