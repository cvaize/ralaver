use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::required::Required;
use crate::{prepare_value, Alert, AppService, AuthService, EmailAddress, EmailMessage, MailService, RandomService, TemplateService, TranslatorService, WebHttpResponse};
use crate::{RateLimitService, WebHttpRequest};
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;
use crate::app::controllers::web::{get_public_context_data, get_public_template_context};

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
    auth_service: Data<AuthService>,
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
    auth_service: Data<AuthService>,
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

    let mut context_data = get_public_context_data(&req, translator_service, app_service);
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.reset_password.title");

    let email_str = translator_service.translate(lang, "page.reset_password.fields.email");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, form_errors, email_errors) = post(
        is_post,
        &req,
        lang,
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
    let mut alerts = req.get_alerts(&translator_service, lang);

    if is_done {
        alerts.push(Alert::success(
            translator_service.translate(lang, "alert.reset_password.success"),
        ));
    }

    let layout_ctx = get_public_template_context(&context_data);
    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.reset_password.header"),
        "back": {
            "label": translator_service.translate(lang, "page.reset_password.back"),
            "href": "/login",
        },
        "form": {
            "action": "/reset-password",
            "method": "post",
            "header": translator_service.translate(lang, "page.reset_password.header"),
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": &data.email,
                    "errors": email_errors,
                }
            ],
            "submit": {
                "label": translator_service.translate(lang, "page.reset_password.submit"),
                "text": translator_service.translate(lang, "page.reset_password.text")
            },
            "errors": form_errors
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

async fn post(
    is_post: bool,
    req: &HttpRequest,
    lang: &str,
    data: &mut Form<ResetPasswordData>,
    email_str: &String,
    translator_service: &TranslatorService,
    auth_service: &AuthService,
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
            .map_err(|_| error::ErrorInternalServerError(""))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RATE_LIMIT_MAX_ATTEMPTS, RATE_LIMIT_TTL)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        if executed {
            email_errors = Required::validated(translator_service, lang, &data.email, |value| {
                Email::validate(translator_service, lang, value, &email_str)
            });
            let email: String = data.email.clone().unwrap_or("".to_string());

            if email_errors.len() == 0 {
                let exists = auth_service
                    .exists_user_by_email(&email)
                    .map_err(|_| error::ErrorInternalServerError(""))?;
                if exists == false {
                    email_errors.push(
                        translator_service.translate(lang, "page.reset_password.validation.email.exists"),
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
                    .map_err(|_| error::ErrorInternalServerError(""))?
                    .to_string();

                let code: String = random_service.str(CODE_LEN);

                auth_service
                    .save_reset_password_code(&email, &code)
                    .map_err(|_| error::ErrorInternalServerError(""))?;

                let params = format!("/reset-password-confirm?code={}&email={}", code, email);
                let button_href = app_service
                    .url()
                    .join(&params)
                    .map_err(|_| error::ErrorInternalServerError(""))?
                    .to_string();

                let ctx = json!({
                    "title": translator_service.translate(lang, "mail.reset_password.title"),
                    "description": translator_service.translate(lang, "mail.reset_password.description"),
                    "lang": lang.to_owned(),
                    "site_name": translator_service.translate(lang, "mail.reset_password.site_name"),
                    "site_href": app_service.url().to_string(),
                    "site_domain": site_domain,
                    "logo_src": logo_src,
                    "header": translator_service.translate(lang, "mail.reset_password.header"),
                    "button_label": translator_service.translate(lang, "mail.reset_password.button"),
                    "button_href": button_href.to_owned(),
                });
                let message = EmailMessage {
                    from: None,
                    reply_to: None,
                    to: EmailAddress { name: None, email },
                    subject: translator_service
                        .translate(lang, "mail.reset_password.subject"),
                    html_body: Some(
                        tmpl_service.render_throw_http("emails/auth/reset_password.hbs", &ctx)?,
                    ),
                    text_body: button_href,
                };
                let send_email_result = mail_service.send_email(&message);

                if send_email_result.is_err() {
                    let email_str =
                        translator_service.translate(lang, "alert.reset_password.fail");
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
                .map_err(|_| error::ErrorInternalServerError(""))?;
            form_errors.push(ttl_message)
        }

        if is_done {
            rate_limit_service
                .clear(&rate_limit_key)
                .map_err(|_| error::ErrorInternalServerError(""))?;
        }
    }

    Ok((is_done, form_errors, email_errors))
}


impl ResetPasswordData {
    pub fn prepare(&mut self) {
        prepare_value!(self.email);
    }
}