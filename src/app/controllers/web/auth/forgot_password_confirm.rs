use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{
    Alert, AlertService, AppService, SessionService, TemplateService, Translator, TranslatorService,
};
use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static FORM_DATA_KEY: &str = "page.forgot_password_confirm.form.data";

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, &translator_service);

    let alerts = app_service.get_ref().alerts(&session);

    let form_data: FormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(LoginForm::empty());

    let fields = form.fields.unwrap_or(Fields::empty());

    let email_field = fields.email.unwrap_or(Field::empty());
    let code_field = fields.code.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    let dark_mode = app_service.get_ref().dark_mode(&req);
    let title_str = translator.simple("auth.page.forgot_password_confirm.title");
    let back_str = translator.simple("auth.page.forgot_password_confirm.back.label");
    let header_str = translator.simple("auth.page.forgot_password_confirm.form.header");
    let email_str = translator.simple("auth.page.forgot_password_confirm.form.fields.email.label");
    let password_str =
        translator.simple("auth.page.forgot_password_confirm.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.forgot_password_confirm.form.fields.confirm_password.label");
    let submit_str = translator.simple("auth.page.forgot_password_confirm.form.submit.label");

    let ctx = json!({
        "title": title_str,
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": dark_mode,
        "back": {
            "label": back_str,
            "href": "/forgot-password",
        },
        "form": {
            "action": "/forgot-password-confirm",
            "method": "post",
            "header": header_str,
            "fields": [
                {
                    "name": "code",
                    "type": "hidden",
                    "value": code_field.value,
                },
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
            "errors": form.errors,
        },
    });

    let s = tmpl.get_ref().render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
pub async fn confirm(
    req: HttpRequest,
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<ForgotPasswordConfirmData>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let data: &ForgotPasswordConfirmData = data.deref();

    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, &translator_service);
    let email_str = translator.simple("auth.page.forgot_password_confirm.form.fields.email.label");
    let password_str =
        translator.simple("auth.page.forgot_password_confirm.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.forgot_password_confirm.form.fields.confirm_password.label");

    let email_errors: Vec<String> = Email::validate(&translator, &data.email, &email_str);
    let code_errors: Vec<String> =
        MinMaxLengthString::validate(&translator, &data.code, 4, 254, "code");
    let password_errors: Vec<String> =
        MinMaxLengthString::validate(&translator, &data.password, 4, 254, &password_str);
    let confirm_password_errors: Vec<String> = MinMaxLengthString::validate(
        &translator,
        &data.confirm_password,
        4,
        254,
        &confirm_password_str,
    );

    if email_errors.len() == 0
        && code_errors.len() == 0
        && password_errors.len() == 0
        && confirm_password_errors.len() == 0
    {
        let alert_str = translator.simple("auth.alert.send_email.success");

        alerts.push(Alert::success(alert_str));
    };

    let email_field = Field {
        value: data.email.to_owned(),
        errors: Some(email_errors),
    };

    let code_field = Field {
        value: data.code.to_owned(),
        errors: Some(code_errors),
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
        code: Some(code_field),
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

    Ok(Redirect::to("/forgot-password-confirm").see_other())
}

#[derive(Deserialize, Debug)]
pub struct ForgotPasswordConfirmData {
    pub code: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
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
    code: Option<Field>,
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
            code: None,
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
