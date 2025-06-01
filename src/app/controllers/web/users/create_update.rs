use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString as MMLS};
use crate::app::validator::rules::required::Required;
use crate::{
    prepare_value, Alert, AlertVariant, AppService, Locale, LocaleService, RateLimitService,
    Session, TemplateService, TranslatableError, TranslatorService, User, UserService,
    UserServiceError, WebAuthService, WebHttpResponse,
};
use actix_web::web::Path;
use actix_web::web::{Data, Form, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "users_create_update";

const ROUTE_NAME: &'static str = "users_create_update";

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub action: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
    pub locale: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
    pub patronymic: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub email: Vec<String>,
    pub password: Vec<String>,
    pub confirm_password: Vec<String>,
    pub locale: Vec<String>,
    pub surname: Vec<String>,
    pub name: Vec<String>,
    pub patronymic: Vec<String>,
}

pub async fn create(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let data = Form(PostData::default());
    invoke(
        None, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s, l_s,
    )
}

pub async fn store(
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    invoke(
        None, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s, l_s,
    )
}

pub async fn edit(
    path: Path<u64>,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let edit_user = u_s.get_ref().first_by_id_throw_http(user_id)?;
    let post_data = PostData {
        _token: None,
        action: None,
        email: Some(edit_user.email.to_owned()),
        password: None,
        confirm_password: None,
        locale: edit_user.locale.to_owned(),
        surname: edit_user.surname.to_owned(),
        name: edit_user.name.to_owned(),
        patronymic: edit_user.patronymic.to_owned(),
    };
    let edit_user = Some(edit_user);
    let data = Form(post_data);
    invoke(
        edit_user, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s, l_s,
    )
}

pub async fn update(
    path: Path<u64>,
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let edit_user = Some(u_s.get_ref().first_by_id_throw_http(user_id)?);
    invoke(
        edit_user, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s, l_s,
    )
}

pub fn invoke(
    edit_user: Option<User>,
    req: HttpRequest,
    mut data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let tr_s = tr_s.get_ref();
    let tm_s = tm_s.get_ref();
    let ap_s = ap_s.get_ref();
    let wa_s = wa_s.get_ref();
    let rl_s = rl_s.get_ref();
    let u_s = u_s.get_ref();
    let l_s = l_s.get_ref();

    //
    let user = user.as_ref();

    let mut alert_variants: Vec<AlertVariant> = Vec::new();
    let mut context_data = get_context_data(ROUTE_NAME, &req, user, &session, tr_s, ap_s, wa_s);

    let lang = &context_data.lang;

    let email_str = tr_s.translate(lang, "page.users.create.fields.email");
    let password_str = tr_s.translate(lang, "page.users.create.fields.password");
    let confirm_password_str = tr_s.translate(lang, "page.users.create.fields.confirm_password");
    let surname_str = tr_s.translate(lang, "page.users.create.fields.surname");
    let name_str = tr_s.translate(lang, "page.users.create.fields.name");
    let patronymic_str = tr_s.translate(lang, "page.users.create.fields.patronymic");
    let locale_str = tr_s.translate(lang, "page.users.create.fields.locale");

    let (title, heading, action) = if let Some(edit_user) = &edit_user {
        let mut vars: HashMap<&str, &str> = HashMap::new();
        let user_name = edit_user.get_full_name_with_id_and_email();
        vars.insert("user_name", &user_name);

        (
            tr_s.variables(lang, "page.users.edit.title", &vars),
            tr_s.variables(lang, "page.users.edit.header", &vars),
            get_edit_url(edit_user.id.to_string().as_str()),
        )
    } else {
        (
            tr_s.translate(lang, "page.users.create.title"),
            tr_s.translate(lang, "page.users.create.header"),
            get_create_url(),
        )
    };

    context_data.title = title;

    //
    let is_post = req.method().eq(&Method::POST);
    let mut is_done = false;
    let mut errors = ErrorMessages::default();

    if is_post {
        wa_s.check_csrf_throw_http(&session, &data._token)?;

        let rate_limit_key = rl_s.make_key_from_request_throw_http(&req, RL_KEY)?;

        let executed = rl_s.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

        if executed {
            errors.email = Required::validated(tr_s, lang, &data.email, |value| {
                Email::validate(tr_s, lang, value, &email_str)
            });

            if edit_user.is_none() {
                errors.password = Required::validated(tr_s, lang, &data.password, |value| {
                    MMLS::validate(tr_s, lang, value, 4, 255, &password_str)
                });
            } else {
                if let Some(password) = &data.password {
                    errors.password = MMLS::validate(tr_s, lang, password, 4, 255, &password_str);
                }
            }

            if edit_user.is_none() || data.password.is_some() {
                errors.confirm_password =
                    Required::validated(tr_s, lang, &data.confirm_password, |value| {
                        MMLS::validate(tr_s, lang, value, 4, 255, &confirm_password_str)
                    });
            }

            if errors.password.len() == 0
                && errors.confirm_password.len() == 0
                && data.password.is_some()
                && data.confirm_password.is_some()
            {
                let mut password_errors2: Vec<String> = Confirmed::validate(
                    tr_s,
                    lang,
                    data.password.as_ref().unwrap(),
                    data.confirm_password.as_ref().unwrap(),
                    &password_str,
                );
                errors.confirm_password.append(&mut password_errors2);
            }

            if let Some(surname) = &data.surname {
                errors.surname = MaxLengthString::validate(tr_s, lang, surname, 255, &surname_str);
            }
            if let Some(name) = &data.name {
                errors.name = MaxLengthString::validate(tr_s, lang, name, 255, &name_str);
            }
            if let Some(patronymic) = &data.patronymic {
                errors.patronymic =
                    MaxLengthString::validate(tr_s, lang, patronymic, 255, &patronymic_str);
            }
            if let Some(locale) = &data.locale {
                errors.locale = MaxLengthString::validate(tr_s, lang, locale, 255, &locale_str);
            }

            if errors.is_empty() {
                let mut password = data.password.to_owned();
                let mut is_need_hash_password = true;
                let id = if let Some(edit_user) = &edit_user {
                    if password.is_none() {
                        is_need_hash_password = false;
                        password = edit_user.password.to_owned();
                    }
                    edit_user.id
                } else {
                    0
                };
                let mut user_data = User::default();
                user_data.id = id;
                user_data.email = data.email.clone().unwrap();
                user_data.password = password;
                user_data.locale = data.locale.to_owned();
                user_data.surname = data.surname.to_owned();
                user_data.name = data.name.to_owned();
                user_data.patronymic = data.patronymic.to_owned();
                let result = u_s.upsert(&mut user_data, is_need_hash_password);

                if let Err(error) = result {
                    if error.eq(&UserServiceError::DuplicateEmail) {
                        errors.email.push(error.translate(lang, tr_s));
                    } else if error.eq(&UserServiceError::PasswordHashFail) {
                        errors.password.push(error.translate(lang, tr_s));
                    } else {
                        errors.form.push(error.translate(lang, tr_s));
                    }
                } else {
                    is_done = true;
                }
            }
        } else {
            let ttl_message = rl_s.ttl_message_throw_http(tr_s, lang, &rate_limit_key)?;
            errors.form.push(ttl_message)
        }

        if is_done {
            rl_s.clear_throw_http(&rate_limit_key)?;
        }
    }

    //
    for form_error in errors.form {
        context_data.alerts.push(Alert::error(form_error));
    }

    if is_done {
        let mut id: String = "".to_string();

        if let Some(edit_user) = &edit_user {
            let user = u_s.first_by_id_throw_http(edit_user.id)?;
            id = user.id.to_string();
            let name_ = user.get_full_name_with_id_and_email();
            alert_variants.push(AlertVariant::UsersUpdateSuccess(name_))
        } else if let Some(email_) = &data.email {
            let user = u_s.first_by_email_throw_http(email_)?;
            id = user.id.to_string();
            let name_ = user.get_full_name_with_id_and_email();
            alert_variants.push(AlertVariant::UsersCreateSuccess(name_))
        }

        if let Some(action) = &data.action {
            if action.eq("save") {
                let url_ = get_edit_url(&id);
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        http::header::LOCATION,
                        http::HeaderValue::from_str(&url_)
                            .map_err(|_| error::ErrorInternalServerError(""))?,
                    ))
                    .finish());
            } else if action.eq("save_and_close") {
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        http::header::LOCATION,
                        http::HeaderValue::from_static("/users"),
                    ))
                    .finish());
            }
        }
    }

    for variant in &alert_variants {
        context_data
            .alerts
            .push(Alert::from_variant(tr_s, lang, variant));
    }

    let default_locale = l_s.get_default_ref();
    let mut locales_: Vec<&Locale> = vec![default_locale];

    for locale_ in context_data.locales {
        if locale_.code.ne(&default_locale.code) {
            locales_.push(locale_);
        }
    }

    let layout_ctx = get_template_context(&context_data);

    let fields = json!({
        "email": { "label": email_str, "value": &data.email, "errors": errors.email },
        "password": { "label": password_str, "value": &data.password, "errors": errors.password },
        "confirm_password": { "label": confirm_password_str, "value": &data.confirm_password, "errors": errors.confirm_password },
        "surname": { "label": surname_str, "value": &data.surname, "errors": errors.surname },
        "name": { "label": name_str, "value": &data.name, "errors": errors.name },
        "patronymic": { "label": patronymic_str, "value": &data.patronymic, "errors": errors.patronymic },
        "locale": { "label": locale_str, "value": &data.locale, "errors": errors.locale, "options": locales_, "placeholder": tr_s.translate(lang, "Not selected..."), }
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "tabs": {
            "main": tr_s.translate(lang, "page.users.create.tabs.main"),
            "extended": tr_s.translate(lang, "page.users.create.tabs.extended"),
        },
        "breadcrumbs": [
            {"href": "/", "label": tr_s.translate(lang, "page.home.header")},
            {"href": "/users", "label": tr_s.translate(lang, "page.users.index.header")},
            {"label": &heading},
        ],
        "form": {
            "action": &action,
            "method": "post",
            "fields": fields,
            "save": tr_s.translate(lang, "Save"),
            "save_and_close": tr_s.translate(lang, "Save and close"),
            "close": {
                "label": tr_s.translate(lang, "Close"),
                "href": "/users"
            },
        },
    });
    let s = tm_s.render_throw_http("pages/users/create-update.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

pub fn get_create_url() -> String {
    "/users/create".to_string()
}

pub fn get_edit_url(id: &str) -> String {
    let mut str_ = "/users/".to_string();
    str_.push_str(id);
    str_
}

impl PostData {
    pub fn prepare(&mut self) {
        prepare_value!(self._token);
        prepare_value!(self.action);
        prepare_value!(self.email);
        prepare_value!(self.password);
        prepare_value!(self.confirm_password);
        prepare_value!(self.locale);
        prepare_value!(self.surname);
        prepare_value!(self.name);
        prepare_value!(self.patronymic);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.len() == 0
            && self.email.len() == 0
            && self.password.len() == 0
            && self.confirm_password.len() == 0
            && self.surname.len() == 0
            && self.name.len() == 0
            && self.patronymic.len() == 0
            && self.locale.len() == 0
    }
}
