use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{AlertVariant, Session, WebHttpRequest, WebHttpResponse};
use crate::{AppService, AuthService, TemplateService, Translator, TranslatorService};
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct LoginData {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    session: Session,
    auth_service: Data<AuthService<'_>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        session,
        Form(LoginData {
            email: None,
            password: None,
        }),
        auth_service,
        tmpl_service,
        app_service,
        translator_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    mut session: Session,
    mut data: Form<LoginData>,
    auth_service: Data<AuthService<'_>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let auth_service = auth_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();

    let user = auth_service.authenticate_by_session(&session);

    if user.is_ok() {
        return Ok(HttpResponse::SeeOther()
            .clear_alerts()
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service);

    let email_str = translator.simple("auth.page.login.form.fields.email.label");
    let password_str = translator.simple("auth.page.login.form.fields.password.label");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, email_errors, password_errors, form_errors) = post(
        is_post,
        &mut data,
        &email_str,
        &password_str,
        &mut session,
        &translator,
        auth_service,
    )
    .await?;

    if is_done {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::LoginSuccess])
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let ctx = json!({
        "title": translator.simple("auth.page.login.title"),
        "locale": locale,
        "locales": locales,
        "alerts": req.get_alerts(&translator),
        "dark_mode": app_service.dark_mode(&req),
        "form": {
            "action": "/login",
            "method": "post",
            "header": translator.simple("auth.page.login.form.header"),
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": &data.email,
                    "errors": email_errors,
                },
                {
                    "label": password_str,
                    "type": "password",
                    "name": "password",
                    "value": &data.password,
                    "errors": password_errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.login.form.submit.label")
            },
            "reset_password": {
                "label": translator.simple("auth.page.login.form.reset_password.label"),
                "href": "/reset-password"
            },
            "register": {
                "label": translator.simple("auth.page.login.form.register.label"),
                "href": "/register"
            },
            "errors": form_errors,
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;

    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type("text/html")
        .body(s))
}

async fn post(
    is_post: bool,
    data: &mut Form<LoginData>,
    email_str: &String,
    password_str: &String,
    session: &mut Session,
    translator: &Translator<'_>,
    auth_service: &AuthService<'_>,
) -> Result<(bool, Vec<String>, Vec<String>, Vec<String>), Error> {
    let mut is_done = false;
    let mut form_errors: Vec<String> = vec![];
    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];

    if is_post {
        email_errors = Required::validated(&translator, &data.email, |value| {
            Email::validate(&translator, value, email_str)
        });
        password_errors = Required::validated(&translator, &data.password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, password_str)
        });

        if email_errors.len() == 0 && password_errors.len() == 0 {
            let email_value = data.email.as_ref().unwrap();
            let password_value = data.password.as_ref().unwrap();
            let auth_result = auth_service.authenticate_by_password(email_value, password_value);

            match auth_result {
                Ok(user_id) => {
                    auth_service
                        .save_user_id_into_session(session, user_id)
                        .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;
                    is_done = true;
                }
                _ => {
                    form_errors.push(translator.simple("auth.alert.login.fail"));
                }
            };
        };

        if let Some(email) = &data.email {
            if email.len() > 400 {
                data.email = None;
            }
        }

        if password_errors.len() != 0 {
            data.password = None;
        }
    }

    Ok((is_done, email_errors, password_errors, form_errors))
}
