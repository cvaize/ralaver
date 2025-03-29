use crate::{Alert, AlertService, AppService, AuthService, Credentials, SessionService, TemplateService, TranslatorService};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use garde::Validate;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;
use std::string::ToString;

static FORM_DATA_KEY: &str = "page.login.form.data";

pub async fn show(
    req: HttpRequest,
    session: Session,
    auth: Data<AuthService>,
    tmpl: Data<TemplateService>,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let user = auth.get_ref().authenticate_by_session(&session);

    if user.is_ok() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/"),
            ))
            .finish());
    }

    let alerts = alert_service
        .get_ref()
        .get_and_remove_from_session(&session)
        .unwrap_or(Vec::new());

    let login_form_data: LoginSessionFormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(LoginSessionFormData::empty());

    let login_form_form = login_form_data.form.unwrap_or(LoginFormForm::empty());

    let login_form_form_fields = login_form_form
        .fields
        .unwrap_or(LoginFormFormFields::empty());

    let login_form_form_fields_email = login_form_form_fields
        .email
        .unwrap_or(LoginFormFormField::empty());

    let login_form_form_fields_password = login_form_form_fields
        .password
        .unwrap_or(LoginFormFormField::empty());

    let dark_mode = app_service.get_ref().get_dark_mode(&req);
    let lang = app_service.get_locale(Some(&req), Some(&session), None);

    let title_str = translator_service.translate(&lang, "auth.page.login.title");

    let ctx = json!({
        "title": title_str,
        "lang": lang,
        "form": {
            "action": "/login",
            "method": "post",
            "header": "Вход",
            "fields": [
                {
                    "label": "E-mail",
                    "type": "email",
                    "name": "email",
                    "value": login_form_form_fields_email.value,
                    "errors": login_form_form_fields_email.errors,
                },
                {
                    "label": "Пароль",
                    "type": "password",
                    "name": "password",
                    "value": login_form_form_fields_password.value,
                    "errors": login_form_form_fields_password.errors,
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
            "errors": login_form_form.errors,
        },
        "alerts": alerts,
        "dark_mode": dark_mode
    });

    let s = tmpl.render_throw_http("pages/auth/login.hbs", &ctx)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<Credentials>,
    auth: Data<AuthService>,
) -> Result<impl Responder, Error> {
    let mut is_redirect_login = true;
    let credentials: &Credentials = data.deref();

    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];
    let mut alerts: Vec<Alert> = vec![];
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
        let auth_result = auth.authenticate_by_credentials(credentials);

        match auth_result {
            Ok(user_id) => {
                auth.insert_user_id_into_session(&session, user_id)
                    .map_err(|_| error::ErrorInternalServerError("Session error"))?;
                is_redirect_login = false;
                alerts.push(Alert::success("Авторизация успешно пройдена.".to_string()));
            }
            _ => {
                form_errors.push("Вход на сайт не был произведен. Возможно, Вы ввели неверное E-mail или пароль.".to_string());
            }
        };
    };

    let email_field = LoginFormFormField {
        value: credentials.email.to_owned(),
        errors: Some(email_errors),
    };

    let password_field = LoginFormFormField {
        value: None,
        errors: Some(password_errors),
    };

    let fields = LoginFormFormFields {
        email: Some(email_field),
        password: Some(password_field),
    };

    let form = LoginFormForm {
        fields: Some(fields),
        errors: Some(form_errors),
    };

    let login_form_data = LoginSessionFormData { form: Some(form) };

    session_service
        .get_ref()
        .insert(&session, FORM_DATA_KEY, &login_form_data)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    if is_redirect_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/").see_other())
    }
}

pub async fn sign_out(
    session: Session,
    auth: Data<AuthService>,
    alert_service: Data<AlertService>,
) -> Result<impl Responder, Error> {
    auth.logout_from_session(&session);

    let alerts = vec![Alert::success("Вы успешно вышли из системы.".to_string())];
    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to("/login").see_other())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSessionFormData {
    form: Option<LoginFormForm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFormForm {
    fields: Option<LoginFormFormFields>,
    errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFormFormFields {
    email: Option<LoginFormFormField>,
    password: Option<LoginFormFormField>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginFormFormField {
    value: Option<String>,
    errors: Option<Vec<String>>,
}

impl LoginSessionFormData {
    fn empty() -> Self {
        Self { form: None }
    }
}

impl LoginFormForm {
    fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

impl LoginFormFormFields {
    fn empty() -> Self {
        Self {
            email: None,
            password: None,
        }
    }
}

impl LoginFormFormField {
    fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}
