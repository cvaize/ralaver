use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{AlertVariant, WebHttpRequest, WebHttpResponse};
use crate::{
    AppService, AuthService, AuthServiceError, Credentials, TemplateService, Translator,
    TranslatorService,
};
use actix_web::web::{Data, Form};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct RegisterData {
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    auth_service: Data<AuthService<'_>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        Form(RegisterData {
            email: None,
            password: None,
            confirm_password: None,
        }),
        tmpl_service,
        app_service,
        translator_service,
        auth_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    mut data: Form<RegisterData>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let auth_service = auth_service.get_ref();

    let (lang, locale, locales) = app_service.locale(Some(&req), None);

    let translator = Translator::new(&lang, translator_service);
    let email_str = translator.simple("auth.page.register.form.fields.email.label");
    let password_str = translator.simple("auth.page.register.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.register.form.fields.confirm_password.label");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, email_errors, password_errors, confirm_password_errors) = post(
        is_post,
        &mut data,
        &email_str,
        &password_str,
        &confirm_password_str,
        &translator,
        auth_service,
    )
    .await?;

    if is_done {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::RegisterSuccess])
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/login"),
            ))
            .finish());
    }

    let ctx = json!({
        "title": translator.simple("auth.page.register.title"),
        "locale": locale,
        "locales": locales,
        "alerts": req.get_alerts(&translator),
        "dark_mode": app_service.dark_mode(&req),
        "form": {
            "action": "/register",
            "method": "post",
            "header": translator.simple("auth.page.register.form.header"),
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
                },
                {
                    "label": confirm_password_str,
                    "type": "password",
                    "name": "confirm_password",
                    "value": &data.confirm_password,
                    "errors": confirm_password_errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.register.form.submit.label"),
            },
            "reset_password": {
                "label": translator.simple("auth.page.register.form.reset_password.label"),
                "href": "/reset-password",
            },
            "login": {
                "label": translator.simple("auth.page.register.form.login.label"),
                "href": "/login",
            },
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
    data: &mut Form<RegisterData>,
    email_str: &String,
    password_str: &String,
    confirm_password_str: &String,
    translator: &Translator<'_>,
    auth_service: &AuthService<'_>,
) -> Result<(bool, Vec<String>, Vec<String>, Vec<String>), Error> {
    let mut is_done = false;
    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];
    let mut confirm_password_errors: Vec<String> = vec![];

    if is_post {
        email_errors = Required::validated(&translator, &data.email, |value| {
            Email::validate(&translator, value, email_str)
        });
        password_errors = Required::validated(&translator, &data.password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, password_str)
        });
        confirm_password_errors =
            Required::validated(&translator, &data.confirm_password, |value| {
                MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
            });

        if password_errors.len() == 0 && confirm_password_errors.len() == 0 {
            let mut password_errors2: Vec<String> = Confirmed::validate(
                &translator,
                data.password.as_ref().unwrap(),
                data.confirm_password.as_ref().unwrap(),
                &password_str,
            );
            confirm_password_errors.append(&mut password_errors2);
        }

        is_done = if email_errors.len() == 0
            && password_errors.len() == 0
            && confirm_password_errors.len() == 0
        {
            let credentials = Credentials {
                email: data.email.to_owned().unwrap(),
                password: data.password.to_owned().unwrap(),
            };

            let register_result = auth_service.register_by_credentials(&credentials);

            if let Err(error) = register_result {
                match error {
                    AuthServiceError::DuplicateEmail => {
                        email_errors.push(translator.simple("auth.alert.register.duplicate"));
                    }
                    _ => {}
                }
                false
            } else {
                true
            }
        } else {
            false
        };

        if let Some(email) = &data.email {
            if email.len() > 400 {
                data.email = None;
            }
        }

        if let Some(password) = &data.password {
            if password.len() > 400 {
                data.password = None;
            }
        }

        if let Some(confirm_password) = &data.confirm_password {
            if confirm_password.len() > 400 {
                data.confirm_password = None;
            }
        }
    }

    Ok((
        is_done,
        email_errors,
        password_errors,
        confirm_password_errors,
    ))
}
