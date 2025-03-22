use crate::app::services::auth::{
    Auth, Credentials, AUTHENTICATED_REDIRECT_TO, NOT_AUTHENTICATED_REDIRECT_TO,
};
use crate::app::services::session::{SessionFlashAlert, SessionFlashData, SessionFlashDataTrait, SessionFlashService, SessionService};
use actix_web::web::Redirect;
use actix_web::{error, web, Error, HttpResponse, Responder, Result};
use garde::Validate;
use handlebars::Handlebars;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;
use std::string::ToString;

static FLASH_DATA_KEY: &str = "page.login";

pub async fn show(
    auth: Auth,
    tmpl: web::Data<Handlebars<'_>>,
    flash_service: SessionFlashService,
    session_service: SessionService
) -> Result<HttpResponse, Error> {
    let user = auth.authenticate_from_session();
    let dark_mode: bool = session_service.dark_mode()
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    if user.is_ok() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static(AUTHENTICATED_REDIRECT_TO),
            ))
            .finish());
    }

    let flash_data: SessionFlashData = flash_service
        .read_and_forget(None)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    let login_flash_data: LoginSessionFlashData = flash_service
        .read_and_forget(Some(FLASH_DATA_KEY))
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    let login_flash_form = login_flash_data.form.unwrap_or(LoginFlashForm::empty());

    let login_flash_form_fields = login_flash_form
        .fields
        .unwrap_or(LoginFlashFormFields::empty());

    let login_flash_form_fields_email = login_flash_form_fields
        .email
        .unwrap_or(LoginFlashFormField::empty());

    let login_flash_form_fields_password = login_flash_form_fields
        .password
        .unwrap_or(LoginFlashFormField::empty());

    let ctx = json!({
        "title": "Вход - Admin panel",
        "lang": "ru",
        "form": {
            "action": "/login",
            "method": "post",
            "header": "Вход",
            "fields": [
                {
                    "label": "E-mail",
                    "type": "email",
                    "name": "email",
                    "value": login_flash_form_fields_email.value,
                    "errors": login_flash_form_fields_email.errors,
                },
                {
                    "label": "Пароль",
                    "type": "password",
                    "name": "password",
                    "value": login_flash_form_fields_password.value,
                    "errors": login_flash_form_fields_password.errors,
                }
            ],
            "submit": {
                "label": "Войти"
            },
            "forgot_password": {
                "label": "Сбросить пароль?",
                "href": "/forgot-password"
            },
            "register": {
                "label": "Зарегистрироваться",
                "href": "/register"
            },
            "errors": login_flash_form.errors,
        },
        "alerts": flash_data.alerts,
        "dark_mode": dark_mode
    });

    let s = tmpl
        .render("pages/auth/login.hbs", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    data: web::Form<Credentials>,
    auth: Auth,
    flash_service: SessionFlashService,
) -> Result<impl Responder, Error> {
    let mut is_redirect_login = true;
    let credentials: &Credentials = data.deref();

    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];
    let mut alerts: Vec<SessionFlashAlert> = vec![];
    let mut form_errors: Vec<String> = vec![];

    if let Err(report) = credentials.validate() {
        for (path, error) in report.iter() {
            if path.to_string() == "email" {
                email_errors.push(error.message().to_string());
            }
            if path.to_string() == "password" {
                password_errors.push(error.message().to_string());
            }
        }
    } else {
        let auth_result = auth.authenticate(credentials);

        match auth_result {
            Ok(user_id) => {
                auth.insert_user_id_into_session(user_id)
                    .map_err(|_| error::ErrorInternalServerError("Session error"))?;
                is_redirect_login = false;
                alerts.push(SessionFlashAlert::Success(
                    "Авторизация успешно пройдена.".to_string(),
                ));
            }
            _ => {
                form_errors.push("Вход на сайт не был произведен. Возможно, Вы ввели неверное E-mail или пароль.".to_string());
            }
        };
    };

    let email_field = LoginFlashFormField {
        value: credentials.email.to_owned(),
        errors: Some(email_errors),
    };

    let password_field = LoginFlashFormField {
        value: None,
        errors: Some(password_errors),
    };

    let fields = LoginFlashFormFields {
        email: Some(email_field),
        password: Some(password_field),
    };

    let form = LoginFlashForm {
        fields: Some(fields),
        errors: Some(form_errors),
    };

    let login_flash_data = LoginSessionFlashData { form: Some(form) };

    let flash_data = SessionFlashData {
        alerts: Some(alerts),
    };

    flash_service
        .save(&flash_data, None)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    flash_service
        .save(&login_flash_data, Some(FLASH_DATA_KEY))
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    if is_redirect_login {
        Ok(Redirect::to(NOT_AUTHENTICATED_REDIRECT_TO).see_other())
    } else {
        Ok(Redirect::to(AUTHENTICATED_REDIRECT_TO).see_other())
    }
}

pub async fn sign_out(
    auth: Auth,
    flash_service: SessionFlashService,
) -> Result<impl Responder, Error> {
    auth.logout();

    let flash_data = SessionFlashData {
        alerts: Some(vec![SessionFlashAlert::Success(
            "Вы успешно вышли из системы.".to_string(),
        )]),
    };

    flash_service
        .save(&flash_data, None)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to(NOT_AUTHENTICATED_REDIRECT_TO).see_other())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSessionFlashData {
    form: Option<LoginFlashForm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFlashForm {
    fields: Option<LoginFlashFormFields>,
    errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFlashFormFields {
    email: Option<LoginFlashFormField>,
    password: Option<LoginFlashFormField>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFlashFormField {
    value: Option<String>,
    errors: Option<Vec<String>>,
}

impl SessionFlashDataTrait for LoginSessionFlashData {
    fn empty() -> Self {
        Self { form: None }
    }
}

impl LoginFlashForm {
    fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

impl LoginFlashFormFields {
    fn empty() -> Self {
        Self {
            email: None,
            password: None,
        }
    }
}

impl LoginFlashFormField {
    fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}
