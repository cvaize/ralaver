use crate::app::controllers::web::{DefaultFields, DefaultForm, Field, FormData};
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{model_redis_impl, Session};
use crate::{
    Alert, AppService, AuthService, Credentials, KeyValueService, SessionService, TemplateService,
    Translator, TranslatorService, ALERTS_KEY,
};
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::Deserialize;
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.login.form.data";

model_redis_impl!(FormData<DefaultFields>);

pub async fn show(
    req: HttpRequest,
    session: Session,
    auth_service: Data<AuthService<'_>>,
    tmpl_service: Data<TemplateService>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let auth_service = auth_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let session_service = session_service.get_ref();
    let key_value_service = key_value_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();

    let user = auth_service.authenticate_by_session(&session);

    if user.is_ok() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service);

    let key = session_service.make_session_data_key(&session, DATA_KEY);
    let form_data: FormData<DefaultFields> = key_value_service
        .get_del(&key)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());
    let fields = form.fields.unwrap_or(DefaultFields::empty());
    let email = fields.email.unwrap_or(Field::empty());
    let password = fields.password.unwrap_or(Field::empty());

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);
    let alerts: Vec<Alert> = key_value_service
        .get_del(&key)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?
        .unwrap_or(vec![]);

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
        "alerts": alerts,
        "dark_mode": app_service.dark_mode(&req)
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
    req: HttpRequest,
    mut session: Session,
    data: Form<LoginData>,
    session_service: Data<SessionService>,
    auth_service: Data<AuthService<'_>>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    key_value_service: Data<KeyValueService>,
) -> Result<impl Responder, Error> {
    let auth_service = auth_service.get_ref();
    let app_service = app_service.get_ref();
    let session_service = session_service.get_ref();
    let translator_service = translator_service.get_ref();
    let key_value_service = key_value_service.get_ref();

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let mut is_redirect_login = true;
    let data: &LoginData = data.deref();

    let translator = Translator::new(&lang, translator_service);

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

    let mut is_valid = email_errors.len() == 0 && password_errors.len() == 0;

    if is_valid {
        let credentials = Credentials {
            email: data.email.clone().unwrap(),
            password: data.password.clone().unwrap(),
        };
        let auth_result = auth_service.authenticate_by_credentials(&credentials);

        match auth_result {
            Ok(user_id) => {
                auth_service.save_user_id_into_session(&mut session, user_id)
                    .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

                is_redirect_login = false;
                let user = auth_service.authenticate_by_session(&session);

                let (lang, _, _) = match user {
                    Ok(user) => app_service.locale(Some(&req), Some(&session), Some(&user)),
                    _ => app_service.locale(Some(&req), Some(&session), None),
                };
                let translator = Translator::new(&lang, translator_service);
                let alert_str = translator.simple("auth.alert.sign_in.success");

                alerts.push(Alert::success(alert_str));
            }
            _ => {
                let alert_str = translator.simple("auth.alert.sign_in.fail");
                form_errors.push(alert_str);
                is_valid = false;
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

    let key = session_service.make_session_data_key(&session, DATA_KEY);
    if is_valid {
        key_value_service
            .del(key)
            .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;
    } else {
        key_value_service
            .set_ex(key, &form_data, 600)
            .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;
    }

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);

    key_value_service
        .set_ex(key, &alerts, 600)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;

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
