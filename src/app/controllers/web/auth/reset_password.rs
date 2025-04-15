use crate::app::controllers::web::{DefaultForm, Field, FormData};
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::required::Required;
use crate::{
    Alert, AlertService, AppService, AuthService, EmailAddress, EmailMessage, MailService,
    RandomService, SessionService, TemplateService, Translator, TranslatorService,
};
use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.reset_password.form.data";
pub static CODE_LEN: usize = 64;

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service.get_ref());

    let alerts = app_service.get_ref().alerts(&session);

    let form_data: FormData<ResetPasswordFields> = session_service
        .get_and_remove(&session, DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());
    let fields = form.fields.unwrap_or(ResetPasswordFields::empty());
    let email_field = fields.email.unwrap_or(Field::empty());

    let ctx = json!({
        "title": translator.simple("auth.page.reset_password.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.get_ref().dark_mode(&req),
        "back": {
            "label": translator.simple("auth.page.reset_password.back.label"),
            "href": "/login",
        },
        "form": {
            "action": "/reset-password",
            "method": "post",
            "header": translator.simple("auth.page.reset_password.form.header"),
            "fields": [
                {
                    "label": translator.simple("auth.page.reset_password.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "value": email_field.value,
                    "errors": email_field.errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.reset_password.form.submit.label"),
                "text": translator.simple("auth.page.reset_password.form.submit.text")
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
    data: Form<ResetPasswordData>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    mail_service: Data<MailService>,
    tmpl: Data<TemplateService>,
    auth_service: Data<AuthService<'_>>,
    random_service: Data<RandomService>,
) -> Result<impl Responder, Error> {
    let auth_service = auth_service.get_ref();
    let data: &ResetPasswordData = data.deref();

    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service.get_ref());
    let email_str = translator.simple("auth.page.reset_password.form.fields.email.label");

    let mut email_errors: Vec<String> = Required::validated(&translator, &data.email, |value| {
        Email::validate(&translator, value, &email_str)
    });
    let email: String = data.email.clone().unwrap_or("".to_string());

    if email_errors.len() == 0 {
        let exists = auth_service
            .exists_user_by_email(&email)
            .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;
        if exists == false {
            email_errors.push(translator.simple("validation.custom.email.exists"));
        }
    }

    if email_errors.len() == 0 {
        let site_domain = app_service
            .url()
            .domain()
            .unwrap_or("localhost")
            .to_string();
        let logo_src = app_service
            .url()
            .join("/svg/logo.svg")
            .map_err(|_| error::ErrorInternalServerError("App url error"))?
            .to_string();

        let code: String = random_service.get_ref().str(CODE_LEN);

        auth_service
            .save_reset_password_code(&email, &code)
            .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

        let params = format!("/reset-password-confirm?code={}&email={}", code, email);
        let button_href = app_service
            .url()
            .join(&params)
            .map_err(|_| error::ErrorInternalServerError("App url error"))?
            .to_string();

        let ctx = json!({
            "title": translator.simple("auth.page.reset_password.mail.title"),
            "description": translator.simple("auth.page.reset_password.mail.description"),
            "lang": lang.to_owned(),
            "site_name": translator.simple("auth.page.reset_password.mail.site_name"),
            "site_href": app_service.url().to_string(),
            "site_domain": site_domain,
            "logo_src": logo_src,
            "header": translator.simple("auth.page.reset_password.mail.header"),
            "button_label": translator.simple("auth.page.reset_password.mail.button"),
            "button_href": button_href.to_owned(),
        });
        let message = EmailMessage {
            from: None,
            reply_to: None,
            to: EmailAddress { name: None, email },
            subject: translator.simple("auth.page.reset_password.mail.subject"),
            html_body: Some(
                tmpl.get_ref()
                    .render_throw_http("emails/auth/reset_password.hbs", &ctx)?,
            ),
            text_body: button_href,
        };
        let send_email_result = mail_service.get_ref().send_email(&message);

        if send_email_result.is_err() {
            let email_str = translator.simple("auth.alert.send_email.fail");
            email_errors.push(email_str);
        }
    }

    let is_valid = email_errors.len() == 0;
    if is_valid {
        let alert_str = translator.simple("auth.alert.send_email.success");

        alerts.push(Alert::success(alert_str));
    };

    let form_data = FormData {
        form: Some(DefaultForm {
            fields: Some(ResetPasswordFields {
                email: Some(Field {
                    value: data.email.to_owned(),
                    errors: Some(email_errors),
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

    Ok(Redirect::to("/reset-password").see_other())
}

#[derive(Deserialize, Debug)]
pub struct ResetPasswordData {
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordFields {
    email: Option<Field>,
}

impl ResetPasswordFields {
    fn empty() -> Self {
        Self { email: None }
    }
}
