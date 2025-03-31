use crate::{Alert, AlertService, AppService, SessionService, TemplateService, TranslatorService};
use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use garde::Validate;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static FORM_DATA_KEY: &str = "page.register.form.data";

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    alert_service: Data<AlertService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let lang = app_service.get_locale_code(Some(&req), Some(&session), None);

    let alerts = alert_service
        .get_ref()
        .get_and_remove_from_session(&session)
        .unwrap_or(Vec::new());

    let form_data: FormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(LoginForm::empty());

    let fields = form.fields.unwrap_or(Fields::empty());

    let email_field = fields.email.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    let dark_mode = app_service.get_ref().get_dark_mode(&req);
    let title_str = translator_service.translate(&lang, "auth.page.register.title");
    let header_str = translator_service.translate(&lang, "auth.page.register.form.header");
    let email_str = translator_service.translate(&lang, "auth.page.register.form.fields.email.label");
    let password_str =
        translator_service.translate(&lang, "auth.page.register.form.fields.password.label");
    let confirm_password_str =
        translator_service.translate(&lang, "auth.page.register.form.fields.confirm_password.label");
    let submit_str = translator_service.translate(&lang, "auth.page.register.form.submit.label");
    let forgot_password_str =
        translator_service.translate(&lang, "auth.page.register.form.forgot_password.label");
    let login_str = translator_service.translate(&lang, "auth.page.register.form.login.label");

    let locale = app_service.get_locale_or_default_ref(&lang);
    let locales = app_service.get_locales_or_default_without_current_ref(&locale.code);

    let ctx = json!({
        "title": title_str,
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": dark_mode,
        "form": {
            "action": "/register",
            "method": "post",
            "header": header_str,
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": email_field.value,
                    "errors": email_field.errors,
                },
                {
                    "label": password_str,
                    "type": "password",
                    "name": "password",
                    "value": password_field.value,
                    "errors": password_field.errors,
                },
                {
                    "label": confirm_password_str,
                    "type": "password",
                    "name": "confirm_password",
                    "value": confirm_password_field.value,
                    "errors": confirm_password_field.errors,
                }
            ],
            "submit": {
                "label": submit_str,
            },
            "forgot_password": {
                "label": forgot_password_str,
                "href": "/forgot-password",
            },
            "login": {
                "label": login_str,
                "href": "/login",
            },
            "errors": form.errors,
        },
    });

    let s = tmpl.get_ref().render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
pub async fn register(
    req: HttpRequest,
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<ForgotPasswordConfirmData>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let data: &ForgotPasswordConfirmData = data.deref();

    let mut email_errors: Vec<String> = vec![];
    let mut password_errors: Vec<String> = vec![];
    let mut confirm_password_errors: Vec<String> = vec![];
    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    if let Err(report) = data.validate() {
        for (path, error) in report.iter() {
            if path.to_string() == "email" {
                email_errors.push(error.message().to_string());
            }
            if path.to_string() == "password" {
                password_errors.push(error.message().to_string());
            }
            if path.to_string() == "confirm_password" {
                confirm_password_errors.push(error.message().to_string());
            }
        }
    } else {
        let lang = app_service.get_locale_code(Some(&req), Some(&session), None);
        let alert_str = translator_service.translate(&lang, "auth.alert.register.success");

        alerts.push(Alert::success(alert_str));
    };

    let email_field = Field {
        value: data.email.to_owned(),
        errors: Some(email_errors),
    };

    let password_field = Field {
        value: data.password.to_owned(),
        errors: Some(password_errors),
    };

    let confirm_password_field = Field {
        value: data.confirm_password.to_owned(),
        errors: Some(confirm_password_errors),
    };

    let fields = Fields {
        email: Some(email_field),
        password: Some(password_field),
        confirm_password: Some(confirm_password_field),
    };

    let form = LoginForm {
        fields: Some(fields),
        errors: Some(form_errors),
    };

    let form_data = FormData { form: Some(form) };

    session_service
        .get_ref()
        .insert(&session, FORM_DATA_KEY, &form_data)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to("/register").see_other())
}

#[derive(Validate, Deserialize, Debug)]
pub struct ForgotPasswordConfirmData {
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub email: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub password: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub confirm_password: Option<String>,
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
    confirm_password: Option<Field>,
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
            confirm_password: None,
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
