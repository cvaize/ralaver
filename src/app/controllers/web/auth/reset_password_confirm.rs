use crate::app::controllers::web::auth::reset_password::CODE_LEN;
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{AlertVariant, WebHttpRequest, WebHttpResponse};
use crate::{AppService, AuthService, TemplateService, Translator, TranslatorService};
use actix_web::web::{Data, Form, Query};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ResetPasswordConfirmQuery {
    pub email: Option<String>,
    pub code: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResetPasswordConfirmData {
    pub code: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    query: Query<ResetPasswordConfirmQuery>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        query,
        Form(ResetPasswordConfirmData {
            code: None,
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
    query: Query<ResetPasswordConfirmQuery>,
    mut data: Form<ResetPasswordConfirmData>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let auth_service = auth_service.get_ref();

    let query = query.into_inner();
    let (lang, locale, locales) = app_service.locale(Some(&req), None);
    let translator = Translator::new(&lang, translator_service);

    let email_str = translator.simple("auth.page.reset_password_confirm.form.fields.email.label");
    let password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label");

    if let Some(email) = query.email {
        data.email = Some(email.to_owned());
    }
    if let Some(code) = query.code {
        data.code = Some(code.to_owned());
    }

    let mut email = data.email.to_owned().unwrap_or("".to_string());
    let mut code = data.code.to_owned().unwrap_or("".to_string());

    if email.len() > 400 {
        email = "".to_string();
    }
    if code.len() != CODE_LEN {
        code = "".to_string();
    }

    let action = format!("/reset-password-confirm?code={}&email={}", code, email);

    let is_post = req.method().eq(&Method::POST);
    let (is_done, email_errors, password_errors, confirm_password_errors, code_errors) = post(
        is_post,
        &mut data,
        &email_str,
        &password_str,
        &confirm_password_str,
        &translator,
        &auth_service,
    )
    .await?;

    if is_done {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::ResetPasswordConfirmSuccess])
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/login"),
            ))
            .finish());
    }

    if code_errors.len() != 0 || data.email.is_none() || data.code.is_none() {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::ResetPasswordConfirmCodeNotEqual])
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/reset-password"),
            ))
            .finish());
    }

    let ctx = json!({
        "title": translator.simple("auth.page.reset_password_confirm.title"),
        "locale": locale,
        "locales": locales,
        "alerts": req.get_alerts(&translator),
        "dark_mode": app_service.dark_mode(&req),
        "back": {
            "label": translator.simple("auth.page.reset_password_confirm.back.label"),
            "href": "/reset-password",
        },
        "form": {
            "action": action,
            "method": "post",
            "header": translator.simple("auth.page.reset_password_confirm.form.header"),
            "fields": [
                {
                    "name": "code",
                    "type": "hidden",
                    "value": code,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "readonly": "readonly",
                    "value": &data.email,
                    "errors": email_errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.password.label"),
                    "type": "password",
                    "name": "password",
                    "value": &data.password,
                    "errors": password_errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label"),
                    "type": "password",
                    "name": "confirm_password",
                    "value": &data.confirm_password,
                    "errors": confirm_password_errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.reset_password_confirm.form.submit.label"),
            },
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn post(
    is_post: bool,
    data: &mut Form<ResetPasswordConfirmData>,
    email_str: &String,
    password_str: &String,
    confirm_password_str: &String,
    translator: &Translator<'_>,
    auth_service: &AuthService<'_>,
) -> Result<(bool, Vec<String>, Vec<String>, Vec<String>, Vec<String>), Error> {
    let mut is_done = false;
    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];
    let mut confirm_password_errors: Vec<String> = vec![];
    let mut code_errors: Vec<String> = vec![];

    if is_post {
        email_errors = Required::validated(&translator, &data.email, |value| {
            Email::validate(&translator, value, &email_str)
        });
        password_errors = Required::validated(&translator, &data.password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, &password_str)
        });
        confirm_password_errors =
            Required::validated(&translator, &data.confirm_password, |value| {
                MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
            });
        code_errors = Required::validated(&translator, &data.code, |value| {
            MinMaxLengthString::validate(&translator, value, CODE_LEN, CODE_LEN, "code")
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
            && code_errors.len() == 0
        {
            let mut is_done1 = false;
            let d_ = "".to_string();
            let email = data.email.as_ref().unwrap_or(&d_);
            let code = data.code.as_ref().unwrap_or(&d_);
            let password = data.password.as_ref().unwrap_or(&d_);

            let is_code_equal: bool = auth_service
                .is_equal_reset_password_code(email, code)
                .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

            if is_code_equal {
                auth_service
                    .update_password_by_email(email, password)
                    .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

                is_done1 = true;
            }

            is_done1
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

        if let Some(code) = &data.code {
            if code.len() > 400 {
                data.code = None;
            }
        }
    }

    Ok((
        is_done,
        email_errors,
        password_errors,
        confirm_password_errors,
        code_errors,
    ))
}
