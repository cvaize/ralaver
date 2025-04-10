use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{
    Alert, AlertService, AppService, AuthService, Credentials, SessionService, TemplateService,
    Translator, TranslatorService,
};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static FORM_DATA_KEY: &str = "page.login.form.data";

pub async fn show(
    req: HttpRequest,
    session: Session,
    auth: Data<AuthService<'_>>,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let user = auth.get_ref().authenticate_by_session(&session);

    if user.is_ok() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let dark_mode = app_service.get_ref().dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, &translator_service);

    let title_str = translator.simple("auth.page.login.title");
    let form_header_str = translator.simple("auth.page.login.form.header");
    let email_str = translator.simple("auth.page.login.form.fields.email.label");
    let password_str = translator.simple("auth.page.login.form.fields.password.label");
    let submit_str = translator.simple("auth.page.login.form.submit.label");
    let forgot_password_str = translator.simple("auth.page.login.form.forgot_password.label");
    let register_str = translator.simple("auth.page.login.form.register.label");

    let alerts = app_service.get_ref().alerts(&session);

    let form_data: FormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(LoginForm::empty());

    let form_fields = form.fields.unwrap_or(Fields::empty());

    let form_fields_email = form_fields.email.unwrap_or(Field::empty());

    let form_fields_password = form_fields.password.unwrap_or(Field::empty());

    let ctx = json!({
        "title": title_str,
        "locale": locale,
        "locales": locales,
        "form": {
            "action": "/login",
            "method": "post",
            "header": form_header_str,
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": form_fields_email.value,
                    "errors": form_fields_email.errors,
                },
                {
                    "label": password_str,
                    "type": "password",
                    "name": "password",
                    "value": form_fields_password.value,
                    "errors": form_fields_password.errors,
                }
            ],
            "submit": {
                "label": submit_str
            },
            "forgot_password": {
                "label": forgot_password_str,
                "href": "/forgot-password"
            },
            "register": {
                "label": register_str,
                "href": "/register"
            },
            "errors": form.errors,
        },
        "alerts": alerts,
        "dark_mode": dark_mode
    });

    let s = tmpl.render_throw_http("pages/auth.hbs", &ctx)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    req: HttpRequest,
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<LoginData>,
    auth: Data<AuthService<'_>>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let mut is_redirect_login = true;
    let data: &LoginData = data.deref();

    let translator = Translator::new(&lang, &translator_service);
    let email_str = translator.simple("auth.page.login.form.fields.email.label");
    let password_str = translator.simple("auth.page.login.form.fields.password.label");

    let email_errors: Vec<String> = match &data.email {
        Some(value) => Email::validate(&translator, value, &email_str),
        None => Required::validate(&translator, &data.email),
    };

    let password_errors: Vec<String> = match &data.password {
        Some(value) => MinMaxLengthString::validate(&translator, value, 4, 255, &password_str),
        None => Required::validate(&translator, &data.password),
    };

    let mut alerts: Vec<Alert> = vec![];
    let mut form_errors: Vec<String> = vec![];

    let is_valid = email_errors.len() == 0 && password_errors.len() == 0;

    if is_valid {
        let credentials = Credentials {
            email: data.email.clone().unwrap(),
            password: data.password.clone().unwrap(),
        };
        let auth_result = auth.authenticate_by_credentials(&credentials);

        match auth_result {
            Ok(user_id) => {
                auth.insert_user_id_into_session(&session, user_id)
                    .map_err(|_| error::ErrorInternalServerError("Session error"))?;
                is_redirect_login = false;
                let user = auth.get_ref().authenticate_by_session(&session);
                let (lang, _, _) = match user {
                    Ok(user) => app_service.locale(Some(&req), Some(&session), Some(&user)),
                    _ => app_service.locale(Some(&req), Some(&session), None),
                };
                let translator = Translator::new(&lang, &translator_service);
                let alert_str = translator.simple("auth.alert.sign_in.success");

                alerts.push(Alert::success(alert_str));
            }
            _ => {
                let alert_str = translator.simple("auth.alert.sign_in.fail");
                form_errors.push(alert_str);
            }
        };
    };

    let email_field = Field {
        value: data.email.to_owned(),
        errors: Some(email_errors),
    };

    let password_field = Field {
        value: None,
        errors: Some(password_errors),
    };

    let fields = Fields {
        email: Some(email_field),
        password: Some(password_field),
    };

    let form = LoginForm {
        fields: Some(fields),
        errors: Some(form_errors),
    };

    let form_data = FormData { form: Some(form) };

    if is_valid {
        session_service.get_ref().remove(&session, FORM_DATA_KEY);
    } else {
        session_service
            .get_ref()
            .insert(&session, FORM_DATA_KEY, &form_data)
            .map_err(|_| error::ErrorInternalServerError("Session error"))?;
    }

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
    req: HttpRequest,
    session: Session,
    auth: Data<AuthService<'_>>,
    alert_service: Data<AlertService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let user = auth.get_ref().authenticate_by_session(&session);
    auth.logout_from_session(&session);

    let (lang, _, _) = match user {
        Ok(user) => app_service.locale(Some(&req), Some(&session), Some(&user)),
        _ => app_service.locale(Some(&req), Some(&session), None),
    };

    let translator = Translator::new(&lang, &translator_service);
    let alert_str = translator.simple("auth.alert.sign_out.success");

    let alerts = vec![Alert::success(alert_str)];
    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to("/login").see_other())
}

#[derive(Deserialize, Debug)]
pub struct LoginData {
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FormData {
    form: Option<LoginForm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginForm {
    fields: Option<Fields>,
    errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fields {
    email: Option<Field>,
    password: Option<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    value: Option<String>,
    errors: Option<Vec<String>>,
}

impl FormData {
    fn empty() -> Self {
        Self { form: None }
    }
}

impl LoginForm {
    fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

impl Fields {
    fn empty() -> Self {
        Self {
            email: None,
            password: None,
        }
    }
}

impl Field {
    fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}
