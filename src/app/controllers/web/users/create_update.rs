use crate::app::controllers::web::profile::{
    get_url as get_profile_url,
};
use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString as MMLS};
use crate::app::validator::rules::required::Required;
use crate::libs::actix_web::types::form::Form;
use crate::{
    prepare_value, Alert, AlertVariant, AppService, Locale, LocaleService, RateLimitService, Role,
    RoleService, Session, TemplateService, TranslatableError, TranslatorService, User, UserColumn,
    UserPolicy, UserService, UserServiceError, WebAuthService, WebHttpResponse,
};
use actix_web::{web::{Path, Data, ReqData}, error, Error, HttpRequest, HttpResponse, Result, http::{Method, header::{LOCATION}}};
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use actix_web::http::header::HeaderValue;

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "users_create_update";

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
    pub roles_ids: Option<Vec<u64>>,
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
    pub roles_ids: Vec<String>,
}

pub async fn create(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let data = Form(PostData::default());
    let user_roles = role_service.get_all_throw_http()?;
    if !UserPolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
        false,
        None,
        req,
        data,
        user,
        user_roles,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
    )
}

pub async fn store(
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !UserPolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
        false,
        None,
        req,
        data,
        user,
        user_roles,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
    )
}

pub async fn edit(
    path: Path<u64>,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !UserPolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let user_id = path.into_inner();
    let edit_user = user_service.get_ref().first_by_id_throw_http(user_id)?;
    let post_data = post_data_from_user(&edit_user);
    let edit_user = Some(edit_user);
    let data = Form(post_data);
    invoke(
        false,
        edit_user,
        req,
        data,
        user,
        user_roles,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
    )
}

pub async fn update(
    path: Path<u64>,
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !UserPolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let user_id = path.into_inner();
    let edit_user = Some(user_service.get_ref().first_by_id_throw_http(user_id)?);
    invoke(
        false,
        edit_user,
        req,
        data,
        user,
        user_roles,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
    )
}

pub fn invoke(
    is_profile: bool,
    edit_user: Option<User>,
    req: HttpRequest,
    mut data: Form<PostData>,
    user: ReqData<Arc<User>>,
    user_roles: Vec<Role>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let user_service = user_service.get_ref();
    let locale_service = locale_service.get_ref();
    let role_service = role_service.get_ref();

    //
    let user = user.as_ref();

    let mut alert_variants: Vec<AlertVariant> = Vec::new();
    let mut context_data = get_context_data(
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
        role_service,
    );

    let lang = &context_data.lang;

    let email_str = translator_service.translate(lang, "page.users.create.fields.email");
    let password_str = translator_service.translate(lang, "page.users.create.fields.password");
    let confirm_password_str = translator_service.translate(lang, "page.users.create.fields.confirm_password");
    let surname_str = translator_service.translate(lang, "page.users.create.fields.surname");
    let name_str = translator_service.translate(lang, "page.users.create.fields.name");
    let patronymic_str = translator_service.translate(lang, "page.users.create.fields.patronymic");
    let locale_str = translator_service.translate(lang, "page.users.create.fields.locale");
    let roles_ids_str = translator_service.translate(lang, "page.users.create.fields.roles_ids");

    let (title, heading, action) = if is_profile {
        let title = translator_service.translate(lang, "page.profile.title");
        let heading = translator_service.translate(lang, "page.profile.header");
        let action = get_profile_url();
        (title, heading, action)
    } else {
        let data = if let Some(edit_user) = &edit_user {
            let mut vars: HashMap<&str, &str> = HashMap::new();
            let user_name = edit_user.get_full_name_with_id_and_email();
            vars.insert("user_name", &user_name);

            (
                translator_service.variables(lang, "page.users.edit.title", &vars),
                translator_service.variables(lang, "page.users.edit.header", &vars),
                get_edit_url(edit_user.id.to_string().as_str()),
            )
        } else {
            (
                translator_service.translate(lang, "page.users.create.title"),
                translator_service.translate(lang, "page.users.create.header"),
                get_create_url(),
            )
        };
        data
    };

    context_data.title = title;

    //
    let is_post = req.method().eq(&Method::POST);
    let mut is_done = false;
    let mut errors = ErrorMessages::default();

    if is_post {
        web_auth_service.check_csrf_throw_http(&session, &data._token)?;

        let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

        let executed =
            rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

        if executed {
            errors.email = Required::validated(
                translator_service,
                lang,
                &data.email,
                |value| Email::validate(translator_service, lang, value, &email_str),
                &email_str,
            );

            if edit_user.is_none() {
                errors.password = Required::validated(
                    translator_service,
                    lang,
                    &data.password,
                    |value| MMLS::validate(translator_service, lang, value, 4, 255, &password_str),
                    &password_str,
                );
            } else {
                if let Some(password) = &data.password {
                    errors.password = MMLS::validate(translator_service, lang, password, 4, 255, &password_str);
                }
            }

            if edit_user.is_none() || data.password.is_some() {
                errors.confirm_password = Required::validated(
                    translator_service,
                    lang,
                    &data.confirm_password,
                    |value| MMLS::validate(translator_service, lang, value, 4, 255, &confirm_password_str),
                    &confirm_password_str,
                );
            }

            if errors.password.len() == 0
                && errors.confirm_password.len() == 0
                && data.password.is_some()
                && data.confirm_password.is_some()
            {
                let mut password_errors2: Vec<String> = Confirmed::validate(
                    translator_service,
                    lang,
                    data.password.as_ref().unwrap(),
                    data.confirm_password.as_ref().unwrap(),
                    &password_str,
                );
                errors.confirm_password.append(&mut password_errors2);
            }

            if let Some(surname) = &data.surname {
                errors.surname = MaxLengthString::validate(translator_service, lang, surname, 255, &surname_str);
            }
            if let Some(name) = &data.name {
                errors.name = MaxLengthString::validate(translator_service, lang, name, 255, &name_str);
            }
            if let Some(patronymic) = &data.patronymic {
                errors.patronymic =
                    MaxLengthString::validate(translator_service, lang, patronymic, 255, &patronymic_str);
            }
            if let Some(locale) = &data.locale {
                errors.locale = MaxLengthString::validate(translator_service, lang, locale, 255, &locale_str);
            }

            if errors.is_empty() {
                let id = if let Some(edit_user) = &edit_user {
                    edit_user.id
                } else {
                    0
                };
                let mut user_data = User::default();
                user_data.id = id;
                user_data.email = data.email.to_owned().unwrap();
                user_data.locale = data.locale.to_owned();
                user_data.surname = data.surname.to_owned();
                user_data.name = data.name.to_owned();
                user_data.patronymic = data.patronymic.to_owned();

                let mut columns: Vec<UserColumn> = vec![
                    UserColumn::Email,
                    UserColumn::Locale,
                    UserColumn::Surname,
                    UserColumn::Name,
                    UserColumn::Patronymic,
                ];

                if UserPolicy::can_set_roles(&user, &user_roles) {
                    user_data.roles_ids = data.roles_ids.to_owned();
                    columns.push(UserColumn::RolesIds);
                }

                let columns: Option<Vec<UserColumn>> = Some(columns);

                let result = user_service.upsert(&user_data, &columns);

                if let Err(error) = result {
                    if error.eq(&UserServiceError::DuplicateEmail) {
                        errors.email.push(error.translate(lang, translator_service));
                    } else {
                        errors.form.push(error.translate(lang, translator_service));
                    }
                }

                if let Some(password) = &data.password {
                    let result = if let Some(edit_user) = &edit_user {
                        user_service.update_password_by_id(edit_user.id, password)
                    } else {
                        let email = data.email.to_owned().unwrap();
                        user_service.update_password_by_email(&email, password)
                    };

                    if let Err(error) = result {
                        if error.eq(&UserServiceError::PasswordHashFail) {
                            errors.password.push(error.translate(lang, translator_service));
                        } else {
                            errors.form.push(error.translate(lang, translator_service));
                        }
                    }
                }

                is_done = errors.is_empty();
            }
        } else {
            let ttl_message =
                rate_limit_service.ttl_message_throw_http(translator_service, lang, &rate_limit_key)?;
            errors.form.push(ttl_message)
        }

        if is_done {
            rate_limit_service.clear_throw_http(&rate_limit_key)?;
        }
    }

    //
    for form_error in errors.form {
        context_data.alerts.push(Alert::error(form_error));
    }

    if is_done {
        let mut id: String = "".to_string();

        if let Some(edit_user) = &edit_user {
            let user = user_service.first_by_id_throw_http(edit_user.id)?;
            id = user.id.to_string();
            let name_ = user.get_full_name_with_id_and_email();
            alert_variants.push(AlertVariant::UsersUpdateSuccess(name_))
        } else if let Some(email_) = &data.email {
            let user = user_service.first_by_email_throw_http(email_)?;
            id = user.id.to_string();
            let name_ = user.get_full_name_with_id_and_email();
            alert_variants.push(AlertVariant::UsersCreateSuccess(name_))
        }

        if let Some(action) = &data.action {
            if action.eq("save") {
                let url_ = if is_profile {
                    get_profile_url()
                } else {
                    get_edit_url(&id)
                };
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        LOCATION,
                        HeaderValue::from_str(&url_)
                            .map_err(|_| error::ErrorInternalServerError(""))?,
                    ))
                    .finish());
            } else if action.eq("save_and_close") {
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        LOCATION,
                        HeaderValue::from_static("/users"),
                    ))
                    .finish());
            }
        }
    }

    for variant in &alert_variants {
        context_data
            .alerts
            .push(Alert::from_variant(translator_service, lang, variant));
    }

    let default_locale = locale_service.get_default_ref();
    let mut locales_: Vec<&Locale> = vec![default_locale];

    for locale_ in context_data.locales {
        if locale_.code.ne(&default_locale.code) {
            locales_.push(locale_);
        }
    }

    let layout_ctx = get_template_context(&context_data);

    let mut field_roles_ids: Option<Value> = None;
    if UserPolicy::can_set_roles(&user, &user_roles) {
        let mut roles_options: Vec<Value> = Vec::new();

        for role in &user_roles {
            let mut checked = false;
            if let Some(val) = &data.roles_ids {
                if val.contains(&role.id) {
                    checked = true;
                }
            }
            roles_options.push(json!({
                "label": role.name,
                "value": &role.id,
                "checked": checked
            }));
        }

        field_roles_ids = Some(
            json!({ "label": roles_ids_str, "value": &data.roles_ids, "errors": errors.roles_ids, "options": roles_options, }),
        );
    }

    let fields = json!({
        "email": { "label": email_str, "value": &data.email, "errors": errors.email },
        "password": { "label": password_str, "value": &data.password, "errors": errors.password },
        "confirm_password": { "label": confirm_password_str, "value": &data.confirm_password, "errors": errors.confirm_password },
        "surname": { "label": surname_str, "value": &data.surname, "errors": errors.surname },
        "name": { "label": name_str, "value": &data.name, "errors": errors.name },
        "patronymic": { "label": patronymic_str, "value": &data.patronymic, "errors": errors.patronymic },
        "locale": { "label": locale_str, "value": &data.locale, "errors": errors.locale, "options": locales_, "placeholder": translator_service.translate(lang, "Not selected..."), },
        "roles_ids": field_roles_ids
    });

    let (breadcrumbs, save_and_close, close) = if is_profile {
        let breadcrumbs = json!([
            {"href": "/", "label": translator_service.translate(lang, "page.profile.breadcrumbs.home")},
            {"label": translator_service.translate(lang, "page.profile.breadcrumbs.profile")},
        ]);
        (breadcrumbs, None, None)
    } else {
        let breadcrumbs = json!([
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/users", "label": translator_service.translate(lang, "page.users.index.header")},
            {"label": &heading},
        ]);
        let save_and_close = Some(json!(translator_service.translate(lang, "Save and close")));
        let close = Some(json!({
            "label": translator_service.translate(lang, "Close"),
            "href": "/users"
        }));
        (breadcrumbs, save_and_close, close)
    };

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "tabs": {
            "main": translator_service.translate(lang, "page.users.create.tabs.main"),
            "extended": translator_service.translate(lang, "page.users.create.tabs.extended"),
        },
        "breadcrumbs": breadcrumbs,
        "form": {
            "action": &action,
            "method": "post",
            "fields": fields,
            "save": translator_service.translate(lang, "Save"),
            "save_and_close": save_and_close,
            "close": close,
        },
    });
    let s = template_service.render_throw_http("pages/users/create-update.hbs", &ctx)?;
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

pub fn post_data_from_user(user: &User) -> PostData {
    PostData {
        _token: None,
        action: None,
        email: Some(user.email.to_owned()),
        password: None,
        confirm_password: None,
        locale: user.locale.to_owned(),
        surname: user.surname.to_owned(),
        name: user.name.to_owned(),
        patronymic: user.patronymic.to_owned(),
        roles_ids: user.roles_ids.to_owned(),
    }
}
