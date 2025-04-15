use crate::app::controllers::web::auth::reset_password::CODE_LEN;
use crate::app::controllers::web::{DefaultForm, Field, FormData};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{
    Alert, AlertService, AppService, AuthService, SessionService, TemplateService, Translator,
    TranslatorService,
};
use actix_session::Session;
use actix_web::web::{Data, Form, Query, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.reset_password_confirm.form.data";

#[derive(Deserialize)]
pub struct ResetPasswordConfirmQuery {
    pub email: Option<String>,
    pub code: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    query: Query<ResetPasswordConfirmQuery>,
) -> Result<HttpResponse, Error> {
    let query = query.into_inner();
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service.get_ref());

    let alerts = app_service.get_ref().alerts(&session);

    let form_data: FormData<ResetPasswordConfirmFields> = session_service
        .get_and_remove(&session, DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());

    let fields = form.fields.unwrap_or(ResetPasswordConfirmFields::empty());

    let mut email_field = fields.email.unwrap_or(Field::empty());
    let mut code_field = fields.code.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    if query.email.is_none() || query.code.is_none() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/reset-password"),
            ))
            .finish());
    }

    if let Some(email) = query.email {
        email_field.value = Some(email.to_owned());
    }

    if let Some(code) = query.code {
        code_field.value = Some(code.to_owned());
    }

    let ctx = json!({
        "title": translator.simple("auth.page.reset_password_confirm.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.get_ref().dark_mode(&req),
        "back": {
            "label": translator.simple("auth.page.reset_password_confirm.back.label"),
            "href": "/reset-password",
        },
        "form": {
            "action": "/reset-password-confirm",
            "method": "post",
            "header": translator.simple("auth.page.reset_password_confirm.form.header"),
            "fields": [
                {
                    "name": "code",
                    "type": "hidden",
                    "value": code_field.value,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "readonly": "readonly",
                    "value": email_field.value,
                    "errors": email_field.errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.password.label"),
                    "type": "password",
                    "name": "password",
                    "value": password_field.value,
                    "errors": password_field.errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label"),
                    "type": "password",
                    "name": "confirm_password",
                    "value": confirm_password_field.value,
                    "errors": confirm_password_field.errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.reset_password_confirm.form.submit.label"),
            },
            "errors": form.errors,
        },
    });

    let s = tmpl.get_ref().render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<ResetPasswordConfirmData>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<impl Responder, Error> {
    let data: &ResetPasswordConfirmData = data.deref();

    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service.get_ref());
    let email_str = translator.simple("auth.page.reset_password_confirm.form.fields.email.label");
    let password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label");

    let email_errors: Vec<String> = Required::validated(&translator, &data.email, |value| {
        Email::validate(&translator, value, &email_str)
    });
    let password_errors: Vec<String> = Required::validated(&translator, &data.password, |value| {
        MinMaxLengthString::validate(&translator, value, 4, 255, &password_str)
    });
    let mut confirm_password_errors: Vec<String> =
        Required::validated(&translator, &data.confirm_password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
        });
    let code_errors: Vec<String> = Required::validated(&translator, &data.code, |value| {
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

    let mut is_redirect_to_reset_password = false;
    let mut is_redirect_to_login = false;

    let is_valid = email_errors.len() == 0
        && code_errors.len() == 0
        && password_errors.len() == 0
        && confirm_password_errors.len() == 0;

    let d_ = "".to_string();
    let email = data.email.as_ref().unwrap_or(&d_);
    let code = data.code.as_ref().unwrap_or(&d_);
    let password = data.password.as_ref().unwrap_or(&d_);

    if is_valid {
        let is_code_equal: bool = auth_service
            .is_equal_reset_password_code(email, code)
            .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

        if is_code_equal {
            let alert_str = translator.simple("auth.alert.confirm.success");
            alerts.push(Alert::success(alert_str));

            is_redirect_to_login = true;

            auth_service
                .get_ref()
                .update_password_by_email(email, password)
                .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;
        } else {
            let alert_str = translator.simple("auth.alert.confirm.code_not_equal");
            alerts.push(Alert::error(alert_str));

            is_redirect_to_reset_password = true;
        }
    }

    let form_data = FormData {
        form: Some(DefaultForm {
            fields: Some(ResetPasswordConfirmFields {
                email: Some(Field {
                    value: data.email.to_owned(),
                    errors: Some(email_errors),
                }),
                code: Some(Field {
                    value: data.code.to_owned(),
                    errors: Some(code_errors),
                }),
                password: Some(Field {
                    value: data.password.to_owned(),
                    errors: Some(password_errors),
                }),
                confirm_password: Some(Field {
                    value: data.confirm_password.to_owned(),
                    errors: Some(confirm_password_errors),
                }),
            }),
            errors: Some(form_errors),
        }),
    };

    if is_valid {
        session_service.get_ref().remove(&session, DATA_KEY);
    } else {
        session_service
            .get_ref()
            .insert(&session, DATA_KEY, &form_data)
            .map_err(|_| error::ErrorInternalServerError("Session error"))?;
    }

    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    if is_redirect_to_reset_password {
        Ok(Redirect::to("/reset-password").see_other())
    } else if is_redirect_to_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        let to = format!("/reset-password-confirm?code={}&email={}", code, email);
        Ok(Redirect::to(to).see_other())
    }
}

#[derive(Deserialize, Debug)]
pub struct ResetPasswordConfirmData {
    pub code: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordConfirmFields {
    pub code: Option<Field>,
    pub email: Option<Field>,
    pub password: Option<Field>,
    pub confirm_password: Option<Field>,
}

impl ResetPasswordConfirmFields {
    fn empty() -> Self {
        Self {
            code: None,
            email: None,
            password: None,
            confirm_password: None,
        }
    }
}
