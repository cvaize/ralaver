use crate::app::controllers::web::{DefaultFields, DefaultForm, Field, FormData};
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{model_redis_impl, AuthServiceError, FlashService, Locale, Session, User, ALERTS_KEY};
use crate::{
    Alert, AppService, AuthService, Credentials, TemplateService, Translator, TranslatorService,
};
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::Deserialize;
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.login.form.data";

model_redis_impl!(FormData<DefaultFields>);

pub async fn show(
    req: HttpRequest,
    session: Session,
    flash_service: Data<FlashService>,
    auth_service: Data<AuthService<'_>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let flash_service = flash_service.get_ref();
    let auth_service = auth_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
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

    let form_data: FormData<DefaultFields> = flash_service
        .all_throw_http(&session, DATA_KEY)?
        .unwrap_or(FormData::empty());

    let alerts: Vec<Alert> = flash_service
        .all_throw_http(&session, ALERTS_KEY)?
        .unwrap_or(vec![]);

    let form = form_data.form.unwrap_or(DefaultForm::empty());
    let fields = form.fields.unwrap_or(DefaultFields::empty());
    let email = fields.email.unwrap_or(Field::empty());
    let password = fields.password.unwrap_or(Field::empty());

    let ctx = json!({
        "title": translator.simple("auth.page.login.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.dark_mode(&req),
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
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
    req: HttpRequest,
    mut session: Session,
    data: Form<LoginData>,
    flash_service: Data<FlashService>,
    auth_service: Data<AuthService<'_>>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let flash_service = flash_service.get_ref();
    let auth_service = auth_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();

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
                auth_service
                    .save_user_id_into_session(&mut session, user_id)
                    .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

                is_redirect_login = false;
                let user = auth_service.authenticate_by_session(&session);

                let (lang, _, _) = locale(&user, app_service, &req, &session);

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

    if is_valid {
        flash_service.delete_throw_http(&session, DATA_KEY)?;
    } else {
        flash_service.save_throw_http(&session, DATA_KEY, &form_data)?;
    }

    if alerts.len() == 0 {
        flash_service.delete_throw_http(&session, ALERTS_KEY)?;
    } else {
        flash_service.save_throw_http(&session, ALERTS_KEY, &alerts)?;
    }

    if is_redirect_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/").see_other())
    }
}

pub fn locale<'a>(
    user: &Result<User, AuthServiceError>,
    app_service: &'a AppService,
    req: &'a HttpRequest,
    session: &'a Session,
) -> (String, &'a Locale, &'a Vec<Locale>) {
    match user {
        Ok(user) => app_service.locale(Some(&req), Some(&session), Some(user)),
        _ => app_service.locale(Some(&req), Some(&session), None),
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginData {
    pub email: Option<String>,
    pub password: Option<String>,
}
