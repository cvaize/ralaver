use crate::app::controllers::web::{DefaultForm, Field, FormData};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{model_redis_impl, Session};
use crate::{
    Alert, AppService, AuthService, AuthServiceError, Credentials, KeyValueService, SessionService,
    TemplateService, Translator, TranslatorService, ALERTS_KEY,
};
use actix_web::web::{Data, Form, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.register.form.data";

model_redis_impl!(FormData<RegisterFields>);

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let session_service = session_service.get_ref();
    let key_value_service = key_value_service.get_ref();
    let translator_service = translator_service.get_ref();
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);

    let key = session_service.make_session_data_key(&session, DATA_KEY);
    let form_data: FormData<RegisterFields> = key_value_service
        .get_del(&key)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());

    let fields = form.fields.unwrap_or(RegisterFields::empty());

    let email_field = fields.email.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    let translator = Translator::new(&lang, translator_service);

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);
    let alerts: Vec<Alert> = key_value_service
        .get_del(&key)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?
        .unwrap_or(vec![]);

    let ctx = json!({
        "title": translator.simple("auth.page.register.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.dark_mode(&req),
        "form": {
            "action": "/register",
            "method": "post",
            "header": translator.simple("auth.page.register.form.header"),
            "fields": [
                {
                    "label": translator.simple("auth.page.register.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "value": email_field.value,
                    "errors": email_field.errors,
                },
                {
                    "label": translator.simple("auth.page.register.form.fields.password.label"),
                    "type": "password",
                    "name": "password",
                    "value": password_field.value,
                    "errors": password_field.errors,
                },
                {
                    "label": translator.simple("auth.page.register.form.fields.confirm_password.label"),
                    "type": "password",
                    "name": "confirm_password",
                    "value": confirm_password_field.value,
                    "errors": confirm_password_field.errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.register.form.submit.label"),
            },
            "reset_password": {
                "label": translator.simple("auth.page.register.form.reset_password.label"),
                "href": "/reset-password",
            },
            "login": {
                "label": translator.simple("auth.page.register.form.login.label"),
                "href": "/login",
            },
            "errors": form.errors,
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    data: Form<RegisterData>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    key_value_service: Data<KeyValueService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<impl Responder, Error> {
    let app_service = app_service.get_ref();
    let auth_service = auth_service.get_ref();
    let session_service = session_service.get_ref();
    let key_value_service = key_value_service.get_ref();
    let translator_service = translator_service.get_ref();
    let data: &RegisterData = data.deref();

    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);

    let translator = Translator::new(&lang, translator_service);
    let email_str = translator.simple("auth.page.register.form.fields.email.label");
    let password_str = translator.simple("auth.page.register.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.register.form.fields.confirm_password.label");

    let mut email_errors: Vec<String> = Required::validated(&translator, &data.email, |value| {
        Email::validate(&translator, value, &email_str)
    });
    let password_errors: Vec<String> = Required::validated(&translator, &data.password, |value| {
        MinMaxLengthString::validate(&translator, value, 4, 255, &password_str)
    });
    let mut confirm_password_errors: Vec<String> =
        Required::validated(&translator, &data.confirm_password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
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

    let is_valid =
        email_errors.len() == 0 && password_errors.len() == 0 && confirm_password_errors.len() == 0;

    let is_registered = if is_valid {
        let credentials = Credentials {
            email: data.email.to_owned().unwrap(),
            password: data.password.to_owned().unwrap(),
        };

        let register_result = auth_service.register_by_credentials(&credentials);

        if let Err(error) = register_result {
            match error {
                AuthServiceError::DuplicateEmail => {
                    email_errors.push(translator.simple("auth.alert.register.duplicate"));
                }
                _ => {}
            }
            false
        } else {
            true
        }
    } else {
        false
    };

    let key = session_service.make_session_data_key(&session, DATA_KEY);
    if is_registered {
        let alert_str = translator.simple("auth.alert.register.success");

        alerts.push(Alert::success(alert_str));

        key_value_service
            .del(key)
            .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;
    } else {
        let form_data = FormData {
            form: Some(DefaultForm {
                fields: Some(RegisterFields {
                    email: Some(Field {
                        value: data.email.to_owned(),
                        errors: Some(email_errors),
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

        key_value_service
            .set_ex(key, &form_data, 600)
            .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;
    }

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);
    key_value_service
        .set_ex(key, &alerts, 600)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;

    if is_registered {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/register").see_other())
    }
}

#[derive(Deserialize, Debug)]
pub struct RegisterData {
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterFields {
    email: Option<Field>,
    password: Option<Field>,
    confirm_password: Option<Field>,
}

impl RegisterFields {
    fn empty() -> Self {
        Self {
            email: None,
            password: None,
            confirm_password: None,
        }
    }
}
