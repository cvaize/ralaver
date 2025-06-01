use crate::{AppService, TemplateService, TranslatorService};
use actix_web::dev::ServiceResponse;
use actix_web::http::header;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use std::collections::HashMap;

pub fn default_error_handler<B>(
    ser_res: ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, Error> {
    // split service response into request and response components
    let (req, res) = ser_res.into_parts();

    if let Some(error) = res.error() {
        let url = req.full_url().to_string();
        log::error!("Http error \"{url}\" \"{error}\"");
    }

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

#[allow(dead_code)]
fn get_error_text<B>(mut response: HttpResponse<B>) -> HttpResponse<String> {
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
    );

    let error_message: String = get_error_message(&response);
    let status_code: String = response.status().as_str().to_owned();

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

    let error_message: String = get_error_message(&response);
    let status_code: String = response.status().as_str().to_owned();

    let mut json = "{\"status_code\":".to_string();
    json.push_str(&status_code);
    json.push_str(", \"error_message\":\"");
    json.push_str(&error_message);
    json.push_str("\"}");
    response.set_body(json)
}

fn get_error_html_response<B>(
    request: &HttpRequest,
    mut response: HttpResponse<B>,
) -> HttpResponse<String> {
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
    );

    let mut error_message: String = get_error_message(&response);
    let status_code: String = response.status().as_str().to_owned();

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

    let translator_service = request
        .app_data::<web::Data<TranslatorService>>()
        .map(|t| t.get_ref());

    if translator_service.is_none() {
        return response.set_body(title);
    }
    let translator_service = translator_service.unwrap();

    // TODO: Вывести пользователя на странице ошибок. 404 ошибка вызывает панику при обращении к request.extensions
    // let extensions = request.extensions();
    // let user = extensions.get::<Arc<User>>().map(|u| {u.as_ref()});
    // dbg!(&user);

    let (lang, locale, _) = app_service.locale(Some(&request), None);

    error_message = translator_service.translate(&lang, &error_message);
    title = status_code.to_owned();
    title.push_str(" - ");
    title.push_str(&error_message);

    let dark_mode = app_service.dark_mode(&request).unwrap_or("".to_string());

    let mut context = HashMap::new();
    context.insert("title", &title);
    context.insert("error_message", &error_message);
    context.insert("status_code", &status_code);
    context.insert("lang", &locale.code);
    context.insert("dark_mode", &dark_mode);

    let html = tmpl
        .render("pages/error/default.hbs", &context)
        .unwrap_or_else(|_| title);
    response.set_body(html)
}

fn get_error_message<B>(response: &HttpResponse<B>) -> String {
    let mut error_message: String = response
        .error()
        .map(|e| e.to_string())
        .unwrap_or("".to_string());

    if error_message.len() == 0 {
        error_message = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
    }
    error_message
}
