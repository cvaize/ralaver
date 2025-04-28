use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::required::Required;
use crate::{
    Alert, AppService, AuthService, EmailAddress, EmailMessage, MailService, RandomService,
    TemplateService, TranslatorService, WebHttpResponse,
};
use crate::{RateLimitService, WebHttpRequest};
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

pub static CODE_LEN: usize = 64;

static RATE_LIMIT_MAX_ATTEMPTS: u64 = 5;
static RATE_LIMIT_TTL: u64 = 60;
static RATE_KEY: &str = "reset_password";

#[derive(Deserialize, Debug)]
pub struct ResetPasswordData {
    pub email: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    mail_service: Data<MailService>,
    auth_service: Data<AuthService<'_>>,
    random_service: Data<RandomService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        Form(ResetPasswordData { email: None }),
        tmpl_service,
        app_service,
        translator_service,
        mail_service,
        auth_service,
        random_service,
        rate_limit_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    mut data: Form<ResetPasswordData>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    mail_service: Data<MailService>,
    auth_service: Data<AuthService<'_>>,
    random_service: Data<RandomService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let mail_service = mail_service.get_ref();
    let auth_service = auth_service.get_ref();
    let random_service = random_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();

    let (lang, locale, locales) = app_service.locale(Some(&req), None);
    let email_str =
        translator_service.translate(&lang, "auth.page.reset_password.form.fields.email.label");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, form_errors, email_errors) = post(
        is_post,
        &req,
        &lang,
        &mut data,
        &email_str,
        translator_service,
        auth_service,
        mail_service,
        tmpl_service,
        app_service,
        random_service,
        rate_limit_service,
    )
    .await?;
    let mut alerts = req.get_alerts(&translator_service, &lang);

    if is_done {
        alerts.push(Alert::success(
            translator_service.translate(&lang, "auth.alert.reset_password.success"),
        ));
    }

    let ctx = json!({
        "title": translator_service.translate(&lang, "auth.page.reset_password.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.dark_mode(&req),
        "back": {
            "label": translator_service.translate(&lang, "auth.page.reset_password.back.label"),
            "href": "/login",
        },
        "form": {
            "action": "/reset-password",
            "method": "post",
            "header": translator_service.translate(&lang, "auth.page.reset_password.form.header"),
            "fields": [
                {
                    "label": translator_service.translate(&lang, "auth.page.reset_password.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "value": &data.email,
                    "errors": email_errors,
                }
            ],
            "submit": {
                "label": translator_service.translate(&lang, "auth.page.reset_password.form.submit.label"),
                "text": translator_service.translate(&lang, "auth.page.reset_password.form.submit.text")
            },
            "errors": form_errors
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type("text/html")
        .body(s))
}

async fn post<'a>(
    is_post: bool,
    req: &HttpRequest,
    lang: &str,
    data: &mut Form<ResetPasswordData>,
    email_str: &String,
    translator_service: &TranslatorService,
    auth_service: &AuthService<'a>,
    mail_service: &MailService,
    tmpl_service: &TemplateService,
    app_service: &AppService,
    random_service: &RandomService,
    rate_limit_service: &RateLimitService,
) -> Result<(bool, Vec<String>, Vec<String>), Error> {
    let mut is_done: bool = false;
    let mut form_errors: Vec<String> = Vec::new();
    let mut email_errors: Vec<String> = Vec::new();

    if is_post {
        let rate_limit_key = rate_limit_service
            .make_key_from_request(req, RATE_KEY)
            .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RATE_LIMIT_MAX_ATTEMPTS, RATE_LIMIT_TTL)
            .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;

        if executed {
            email_errors = Required::validated(translator_service, lang, &data.email, |value| {
                Email::validate(translator_service, lang, value, &email_str)
            });
            let email: String = data.email.clone().unwrap_or("".to_string());

            if email_errors.len() == 0 {
                let exists = auth_service
                    .exists_user_by_email(&email)
                    .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;
                if exists == false {
                    email_errors.push(
                        translator_service.translate(&lang, "validation.custom.email.exists"),
                    );
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

                let code: String = random_service.str(CODE_LEN);

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
                    "title": translator_service.translate(&lang, "auth.page.reset_password.mail.title"),
                    "description": translator_service.translate(&lang, "auth.page.reset_password.mail.description"),
                    "lang": lang.to_owned(),
                    "site_name": translator_service.translate(&lang, "auth.page.reset_password.mail.site_name"),
                    "site_href": app_service.url().to_string(),
                    "site_domain": site_domain,
                    "logo_src": logo_src,
                    "header": translator_service.translate(&lang, "auth.page.reset_password.mail.header"),
                    "button_label": translator_service.translate(&lang, "auth.page.reset_password.mail.button"),
                    "button_href": button_href.to_owned(),
                });
                let message = EmailMessage {
                    from: None,
                    reply_to: None,
                    to: EmailAddress { name: None, email },
                    subject: translator_service
                        .translate(&lang, "auth.page.reset_password.mail.subject"),
                    html_body: Some(
                        tmpl_service.render_throw_http("emails/auth/reset_password.hbs", &ctx)?,
                    ),
                    text_body: button_href,
                };
                let send_email_result = mail_service.send_email(&message);

                if send_email_result.is_err() {
                    let email_str =
                        translator_service.translate(&lang, "auth.alert.reset_password.fail");
                    form_errors.push(email_str);
                } else {
                    is_done = true;
                }
            }

            if let Some(email) = &data.email {
                if email.len() > 400 {
                    data.email = None;
                }
            }
        } else {
            let ttl_message = rate_limit_service
                .ttl_message(translator_service, lang, &rate_limit_key)
                .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;
            form_errors.push(ttl_message)
        }

        if is_done {
            rate_limit_service
                .clear(&rate_limit_key)
                .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;
        }
    }

    Ok((is_done, form_errors, email_errors))
}
