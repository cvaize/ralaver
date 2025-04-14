use crate::app::controllers::web::helpers::{DefaultFields, DefaultForm, Field, FormData};
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
use serde_derive::Deserialize;
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.login.form.data";

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

    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service.get_ref());

    let form_data: FormData<DefaultFields> = session_service
        .get_and_remove(&session, DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());
    let fields = form.fields.unwrap_or(DefaultFields::empty());
    let email = fields.email.unwrap_or(Field::empty());
    let password = fields.password.unwrap_or(Field::empty());

    let ctx = json!({
        "title": translator.simple("auth.page.login.title"),
        "locale": locale,
        "locales": locales,
        "form": {
            "action": "/login",
            "method": "post",
            "header": translator.simple("auth.page.login.form.header"),
            "fields": [
                {
                    "label": translator.simple("auth.page.login.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "value": email.value,
                    "errors": email.errors,
                },
                {
                    "label": translator.simple("auth.page.login.form.fields.password.label"),
                    "type": "password",
                    "name": "password",
                    "value": password.value,
                    "errors": password.errors,
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
            "errors": form.errors,
        },
        "alerts": app_service.get_ref().alerts(&session),
        "dark_mode": app_service.get_ref().dark_mode(&req)
    });

    let s = tmpl.render_throw_http("pages/auth.hbs", &ctx)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
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

    let translator = Translator::new(&lang, translator_service.get_ref());

    let email_str = translator.simple("auth.page.login.form.fields.email.label");
    let password_str = translator.simple("auth.page.login.form.fields.password.label");

    let email_errors: Vec<String> = Required::validated(&translator, &data.email, |value| {
        Email::validate(&translator, value, &email_str)
    });
    let password_errors: Vec<String> = Required::validated(&translator, &data.password, |value| {
        MinMaxLengthString::validate(&translator, value, 4, 255, &password_str)
    });

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
                let translator = Translator::new(&lang, translator_service.get_ref());
                let alert_str = translator.simple("auth.alert.sign_in.success");

                alerts.push(Alert::success(alert_str));
            }
            _ => {
                let alert_str = translator.simple("auth.alert.sign_in.fail");
                form_errors.push(alert_str);
            }
        };
    };

    let form_data = FormData {
        form: Some(DefaultForm {
            fields: Some(DefaultFields {
                email: Some(Field {
                    value: data.email.to_owned(),
                    errors: Some(email_errors),
                }),
                password: Some(Field {
                    value: None,
                    errors: Some(password_errors),
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

    if is_redirect_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/").see_other())
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginData {
    pub email: Option<String>,
    pub password: Option<String>,
}
