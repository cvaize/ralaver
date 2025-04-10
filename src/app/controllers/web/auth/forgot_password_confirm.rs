use crate::app::controllers::web::auth::forgot_password::CODE_LEN;
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

static FORM_DATA_KEY: &str = "page.forgot_password_confirm.form.data";

#[derive(Deserialize)]
pub struct ForgotPasswordConfirmQuery {
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
    query: Query<ForgotPasswordConfirmQuery>,
) -> Result<HttpResponse, Error> {
    let query = query.into_inner();
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, &translator_service);

    let alerts = app_service.get_ref().alerts(&session);

    let form_data: FormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(LoginForm::empty());

    let fields = form.fields.unwrap_or(Fields::empty());

    let mut email_field = fields.email.unwrap_or(Field::empty());
    let mut code_field = fields.code.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    if query.email.is_none() || query.code.is_none() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/forgot-password"),
            ))
            .finish());
    }

    if let Some(email) = query.email {
        email_field.value = Some(email.to_owned());
    }

    if let Some(code) = query.code {
        code_field.value = Some(code.to_owned());
    }

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
                    "readonly": "readonly",
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
    auth_service: Data<AuthService<'_>>,
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

    let mut email_errors: Vec<String> = match &data.email {
        Some(value) => Email::validate(&translator, value, &email_str),
        None => Required::validate(&translator, &data.email),
    };

    let mut code_errors: Vec<String> = match &data.code {
        Some(value) => MinMaxLengthString::validate(&translator, value, CODE_LEN, CODE_LEN, "code"),
        None => Required::validate(&translator, &data.code),
    };

    let password_errors: Vec<String> = match &data.password {
        Some(value) => MinMaxLengthString::validate(&translator, value, 4, 255, &password_str),
        None => Required::validate(&translator, &data.password),
    };

    let mut confirm_password_errors: Vec<String> = match &data.confirm_password {
        Some(value) => {
            MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
        }
        None => Required::validate(&translator, &data.confirm_password),
    };

    if password_errors.len() == 0 && confirm_password_errors.len() == 0 {
        let mut password_errors2: Vec<String> = match &data.password {
            Some(password) => match &data.confirm_password {
                Some(confirm_password) => {
                    Confirmed::validate(&translator, password, confirm_password, &password_str)
                }
                None => vec![],
            },
            None => vec![],
        };

        confirm_password_errors.append(&mut password_errors2);
    }

    let mut is_redirect_to_forgot_password = false;
    let mut is_redirect_to_login = false;

    let is_valid = email_errors.len() == 0
        && code_errors.len() == 0
        && password_errors.len() == 0
        && confirm_password_errors.len() == 0;

    if is_valid {
        let d_ = "".to_string();
        let email = data.email.as_ref().unwrap_or(&d_);
        let code = data.code.as_ref().unwrap_or(&d_);
        let password = data.password.as_ref().unwrap_or(&d_);

        let is_code_equal: bool = auth_service
            .is_equal_forgot_password_code(email, code)
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

            is_redirect_to_forgot_password = true;
        }
    }

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

    if is_redirect_to_forgot_password {
        Ok(Redirect::to("/forgot-password").see_other())
    } else if is_redirect_to_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/forgot-password-confirm").see_other())
    }
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
    pub form: Option<LoginForm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginForm {
    pub fields: Option<Fields>,
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fields {
    pub code: Option<Field>,
    pub email: Option<Field>,
    pub password: Option<Field>,
    pub confirm_password: Option<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: Option<String>,
    pub errors: Option<Vec<String>>,
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
