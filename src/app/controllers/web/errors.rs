use crate::{AppService, TemplateService};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{web, Either, Error, HttpRequest, HttpResponse};
use std::collections::HashMap;

pub fn default_error_handler<B>(
    mut ser_res: ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, Error> {
    // let path = ser_res.request().uri().path();

    // split service response into request and response components
    let (req, res) = ser_res.into_parts();

    let path = req.uri().path();
    let res = if path.starts_with("/css") || path.starts_with("/js") || path.starts_with("/svg") {
        res.set_body("".to_string())
    } else {
        if path.starts_with("/api") {
            get_error_json_response(res)
        } else {
            get_error_html_response(&req, res)
        }
    };

    // modified bodies need to be boxed and placed in the "right" slot
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res))
}

// Custom error handlers, to return HTML responses when an error occurs.
pub fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "TEST: Page not found");
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

    let tmpl = request
        .app_data::<web::Data<TemplateService>>()
        .map(|t| t.get_ref());

    if tmpl.is_none() {
        return fallback(error);
    }
    let tmpl = tmpl.unwrap();

    let app_service = request
        .app_data::<web::Data<AppService>>()
        .map(|t| t.get_ref());

    if app_service.is_none() {
        return fallback(error);
    }
    let app_service = app_service.unwrap();

    let (_, locale, _) = app_service.locale(Some(&request), None);
    let lang = locale.code.to_string();
    let dark_mode = app_service.dark_mode(&request).unwrap_or("".to_string());
    let error_message = error.to_owned();
    let status_code = res.status().as_str().to_owned();
    let mut title = status_code.to_owned();
    title.push_str(" - ");
    title.push_str(error);

    let mut context = HashMap::new();
    context.insert("title", title);
    context.insert("error_message", error_message);
    context.insert("status_code", status_code);
    context.insert("lang", lang);
    context.insert("dark_mode", dark_mode);

    let body = tmpl.render("pages/error/default.hbs", &context);

    match body {
        Ok(body) => HttpResponse::build(res.status())
            .content_type(ContentType::html())
            .body(body),
        Err(_) => fallback(error),
    }
}

#[allow(dead_code)]
fn get_error_text<B>(mut response: HttpResponse<B>) -> HttpResponse<String> {
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
    );

    let status_code = response.status().as_str().to_owned();
    let error_message: String = response
        .error()
        .map(|e| e.to_string())
        .unwrap_or("".to_string());
    let mut title = status_code.to_owned();
    title.push_str(" - ");
    title.push_str(&error_message);
    response.set_body(title)
}

fn get_error_json_response<B>(mut response: HttpResponse<B>) -> HttpResponse<String> {
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
    );

    let status_code = response.status().as_str().to_owned();
    let error_message: String = response
        .error()
        .map(|e| e.to_string())
        .unwrap_or("".to_string());

    let mut json = "{\"status_code\":\"".to_string();
    json.push_str(&status_code);
    json.push_str("\", \"error_message\":\"");
    json.push_str(&error_message);
    json.push_str("\"}");
    response.set_body(json)
}

fn get_error_html_response<B>(request: &HttpRequest, mut response: HttpResponse<B>) -> HttpResponse<String> {
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
    );

    let error_message: String = response
        .error()
        .map(|e| e.to_string())
        .unwrap_or("".to_string());

    let status_code = response.status().as_str().to_owned();
    let mut title = status_code.to_owned();
    title.push_str(" - ");
    title.push_str(&error_message);

    let tmpl = request
        .app_data::<web::Data<TemplateService>>()
        .map(|t| t.get_ref());

    if tmpl.is_none() {
        return response.set_body(title);
    }
    let tmpl = tmpl.unwrap();

    let app_service = request
        .app_data::<web::Data<AppService>>()
        .map(|t| t.get_ref());

    if app_service.is_none() {
        return response.set_body(title);
    }

    let app_service = app_service.unwrap();

    let (_, locale, _) = app_service.locale(Some(&request), None);
    let lang = locale.code.to_string();
    let dark_mode = app_service.dark_mode(&request).unwrap_or("".to_string());

    let mut context = HashMap::new();
    context.insert("title", &title);
    context.insert("error_message", &error_message);
    context.insert("status_code", &status_code);
    context.insert("lang", &lang);
    context.insert("dark_mode", &dark_mode);

    let html = tmpl.render("pages/error/default.hbs", &context)
        .unwrap_or_else(|_| title);
    response.set_body(html)
}
