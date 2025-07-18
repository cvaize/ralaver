use crate::app::controllers::web::profile::get_url as get_profile_url;
use crate::app::controllers::web::{get_context_data, get_template_context, ContextData};
use crate::app::validator::rules::bytes_max_length::BytesMaxLength;
use crate::app::validator::rules::bytes_mut_max_length::BytesMutMaxLength;
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::contains_str::ContainsStr;
use crate::app::validator::rules::contains_vec_str::ContainsVecStr;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::required::Required;
use crate::app::validator::rules::str_max_chars_count::StrMaxCharsCount;
use crate::app::validator::rules::str_max_length::StrMaxLength;
use crate::app::validator::rules::str_min_max_chars_count::StrMinMaxCharsCount as MMCC;
use crate::{assign_value_bytes_to_string, prepare_value, Alert, AlertVariant, AppService, FileService, Locale, LocaleService, RateLimitService, Role, RoleService, Session, TemplateService, TranslatableError, TranslatorService, User, UserColumn, UserFileService, UserPolicy, UserService, UserServiceError, WebAuthService, WebHttpResponse, USER_AVATAR_MAX_SIZE, USER_AVATAR_MIMES};
use actix_multipart::Multipart;
use actix_web::http::header::HeaderValue;
use actix_web::{
    error,
    http::{header::LOCATION, Method},
    web::{Data, Path, ReqData},
    Error, HttpRequest, HttpResponse, Result,
};
use bytes::{Bytes, BytesMut};
use futures_util::{StreamExt, TryStreamExt};
use mime::Mime;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use strum::VariantNames;
use strum_macros::{Display, EnumString};
use crate::app::validator::rules::mimes::Mimes;

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    EnumString,
    Serialize,
    Deserialize,
    strum_macros::VariantNames,
    Eq,
    PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    Save,
    SaveAndClose,
}
const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "users_create_update";

#[derive(Default, Debug)]
pub struct Avatar {
    pub bytes: Bytes,
    pub filename: Option<String>,
    pub mime: Option<Mime>,
}

#[derive(Default, Debug)]
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
    pub avatar: Option<Avatar>,
}

#[derive(Default, Debug)]
pub struct ErrorMessages {
    pub form: Vec<String>,
    pub email: Vec<String>,
    pub password: Vec<String>,
    pub confirm_password: Vec<String>,
    pub locale: Vec<String>,
    pub surname: Vec<String>,
    pub name: Vec<String>,
    pub patronymic: Vec<String>,
    pub roles_ids: Vec<String>,
    pub avatar: Vec<String>,
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
            && self.roles_ids.len() == 0
            && self.avatar.len() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeRoute {
    Create,
    Store,
    Edit,
    Update,
    ProfileEdit,
    ProfileUpdate,
}

pub struct InvokeData<'a> {
    pub route: InvokeRoute,
    pub auth_user: &'a User,
    pub auth_session: &'a Session,
    pub entity: Option<User>,
    pub payload: Option<Multipart>,
    pub req: HttpRequest,
    // Service
    pub translator_service: Data<TranslatorService>,
    pub template_service: Data<TemplateService>,
    pub app_service: Data<AppService>,
    pub web_auth_service: Data<WebAuthService>,
    pub rate_limit_service: Data<RateLimitService>,
    pub user_service: Data<UserService>,
    pub locale_service: Data<LocaleService>,
    pub role_service: Data<RoleService>,
    pub user_file_service: Data<UserFileService>,
}

pub async fn create(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    // Services
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    invoke(InvokeData {
        route: InvokeRoute::Create,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: None,
        payload: None,
        req,
        //
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
    })
    .await

    // let data = PostData::default();
    // let errors = ErrorMessages::default();
    // let user_roles = role_service.all_throw_http()?;
    // if !UserPolicy::can_create(&user, &user_roles) {
    //     return Err(error::ErrorForbidden(""));
    // }
    // invoke(
    //     false,
    //     None,
    //     req,
    //     data,
    //     errors,
    //     user,
    //     user_roles,
    //     session,
    //     translator_service,
    //     template_service,
    //     app_service,
    //     web_auth_service,
    //     rate_limit_service,
    //     user_service,
    //     locale_service,
    //     role_service,
    //     user_file_service,
    // ).await
}

pub async fn store(
    req: HttpRequest,
    payload: Multipart,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    // Services
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    invoke(InvokeData {
        route: InvokeRoute::Store,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: None,
        payload: Some(payload),
        req,
        //
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
    })
    .await
    // let user = user.as_ref();
    // let context_data = get_context_data(
    //     &req,
    //     user,
    //     &session,
    //     translator_service.get_ref(),
    //     app_service.get_ref(),
    //     web_auth_service.get_ref(),
    //     role_service.get_ref(),
    // );
    //
    //
    // let mut data = PostData::default();
    // let errors = data.fill_from_multipart(payload, &context_data.lang, translator_service.clone()).await?;
    // let user_roles = role_service.all_throw_http()?;
    // if !UserPolicy::can_create(&user, &user_roles) {
    //     return Err(error::ErrorForbidden(""));
    // }
    // invoke(
    //     false,
    //     None,
    //     req,
    //     data,
    //     errors,
    //     user,
    //     user_roles,
    //     session,
    //     translator_service,
    //     template_service,
    //     app_service,
    //     web_auth_service,
    //     rate_limit_service,
    //     user_service,
    //     locale_service,
    //     role_service,
    //     user_file_service,
    // ).await
}

pub async fn edit(
    path: Path<u64>,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    // Services
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let entity = user_service.get_ref().first_by_id_throw_http(user_id)?;
    invoke(InvokeData {
        route: InvokeRoute::Edit,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: Some(entity),
        payload: None,
        req,
        //
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
    })
    .await
    // let user_roles = role_service.all_throw_http()?;
    // if !UserPolicy::can_update(&user, &user_roles) {
    //     return Err(error::ErrorForbidden(""));
    // }
    // let user_id = path.into_inner();
    // let edit_user = user_service.get_ref().first_by_id_throw_http(user_id)?;
    // let post_data = post_data_from_user(&edit_user);
    // let edit_user = Some(edit_user);
    // let data = Form(post_data);
    // invoke(
    //     false,
    //     edit_user,
    //     req,
    //     data,
    //     user,
    //     user_roles,
    //     session,
    //     translator_service,
    //     template_service,
    //     app_service,
    //     web_auth_service,
    //     rate_limit_service,
    //     user_service,
    //     locale_service,
    //     role_service,
    //     user_file_service,
    // )
}

pub async fn update(
    path: Path<u64>,
    req: HttpRequest,
    payload: Multipart,
    // data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    // Services
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let entity = user_service.get_ref().first_by_id_throw_http(user_id)?;
    invoke(InvokeData {
        route: InvokeRoute::Update,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: Some(entity),
        payload: Some(payload),
        req,
        //
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
    })
    .await
    // let data = Form(PostData::default());
    // let user_roles = role_service.all_throw_http()?;
    // if !UserPolicy::can_update(&user, &user_roles) {
    //     return Err(error::ErrorForbidden(""));
    // }
    // let user_id = path.into_inner();
    // let edit_user = Some(user_service.get_ref().first_by_id_throw_http(user_id)?);
    // invoke(
    //     false,
    //     edit_user,
    //     req,
    //     data,
    //     user,
    //     user_roles,
    //     session,
    //     translator_service,
    //     template_service,
    //     app_service,
    //     web_auth_service,
    //     rate_limit_service,
    //     user_service,
    //     locale_service,
    //     role_service,
    //     user_file_service,
    // )
}

pub async fn invoke(
    invoke_data: InvokeData<'_>, // is_profile: bool,
                                 // edit_user: Option<User>,
                                 // req: HttpRequest,
                                 // mut data: PostData,
                                 // mut errors: ErrorMessages,
                                 // mut context_data: ContextData,
                                 // user: &User,
                                 // user_roles: Vec<Role>,
                                 // session: ReqData<Arc<Session>>,
                                 // translator_service: Data<TranslatorService>,
                                 // template_service: Data<TemplateService>,
                                 // web_auth_service: Data<WebAuthService>,
                                 // rate_limit_service: Data<RateLimitService>,
                                 // user_service: Data<UserService>,
                                 // locale_service: Data<LocaleService>,
                                 // user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    let route: InvokeRoute = invoke_data.route;
    let auth_user: &User = invoke_data.auth_user;
    let auth_session: &Session = invoke_data.auth_session;
    let mut entity: Option<User> = invoke_data.entity;
    let payload: Option<Multipart> = invoke_data.payload;
    let req: HttpRequest = invoke_data.req;
    let is_profile = route.eq(&InvokeRoute::ProfileEdit) || route.eq(&InvokeRoute::ProfileUpdate);

    if is_profile {
        entity = Some(auth_user.to_owned());
    }

    // Services
    let translator_service = invoke_data.translator_service.get_ref();
    let template_service = invoke_data.template_service.get_ref();
    let app_service = invoke_data.app_service.get_ref();
    let web_auth_service = invoke_data.web_auth_service.get_ref();
    let rate_limit_service = invoke_data.rate_limit_service.get_ref();
    let user_service = invoke_data.user_service.get_ref();
    let locale_service = invoke_data.locale_service.get_ref();
    let role_service = invoke_data.role_service.get_ref();
    let user_file_service = invoke_data.user_file_service.get_ref();

    let user_roles = role_service.all_throw_http()?;

    if (route.eq(&InvokeRoute::Create) && !UserPolicy::can_create(auth_user, &user_roles))
        || (route.eq(&InvokeRoute::Store) && !UserPolicy::can_create(auth_user, &user_roles))
        || (route.eq(&InvokeRoute::Edit) && !UserPolicy::can_update(auth_user, &user_roles))
        || (route.eq(&InvokeRoute::Update) && !UserPolicy::can_update(auth_user, &user_roles))
    {
        return Err(error::ErrorForbidden(""));
    }

    //
    let mut alert_variants: Vec<AlertVariant> = Vec::new();
    let mut context_data = get_context_data(
        &req,
        auth_user,
        auth_session,
        translator_service,
        app_service,
        web_auth_service,
        role_service,
    );

    let lang = context_data.lang.as_str();

    let default_locale = locale_service.get_default_ref();
    let mut locales_: Vec<&Locale> = vec![default_locale];
    let mut str_locales: Vec<&str> = Vec::new();

    for locale_ in context_data.locales {
        if locale_.code.ne(&default_locale.code) {
            locales_.push(locale_);
        }
        str_locales.push(locale_.code.as_str());
    }

    // PrepareData
    let email_str = translator_service.translate(lang, "page.users.create.fields.email");
    let password_str = translator_service.translate(lang, "page.users.create.fields.password");
    let confirm_password_str =
        translator_service.translate(lang, "page.users.create.fields.confirm_password");
    let surname_str = translator_service.translate(lang, "page.users.create.fields.surname");
    let name_str = translator_service.translate(lang, "page.users.create.fields.name");
    let patronymic_str = translator_service.translate(lang, "page.users.create.fields.patronymic");
    let locale_str = translator_service.translate(lang, "page.users.create.fields.locale");
    let roles_ids_str = translator_service.translate(lang, "page.users.create.fields.roles_ids");
    let avatar_str = translator_service.translate(lang, "page.users.create.fields.avatar");

    let mut data: PostData = PostData::default();

    if route.eq(&InvokeRoute::Edit) || route.eq(&InvokeRoute::ProfileEdit) {
        if let Some(entity) = &entity {
            data.fill_from_user(entity);
        }
    }

    let mut errors: ErrorMessages = ErrorMessages::default();

    if let Some(payload) = payload {
        errors = data
            .prepare_from_multipart(
                payload,
                errors,
                &entity,
                lang,
                translator_service,
                &email_str,
                &password_str,
                &confirm_password_str,
                &locale_str,
                &surname_str,
                &name_str,
                &patronymic_str,
                &roles_ids_str,
                &avatar_str,
                &str_locales,
            )
            .await?;
    }

    let is_post = route.eq(&InvokeRoute::Store)
        || route.eq(&InvokeRoute::Update)
        || route.eq(&InvokeRoute::ProfileUpdate);

    if is_post {
        web_auth_service.check_csrf_throw_http(&auth_session, &data._token)?;
    }

    let (title, heading, action) = match &route {
        InvokeRoute::Create | InvokeRoute::Store | InvokeRoute::Edit | InvokeRoute::Update => {
            let data = if let Some(entity) = &entity {
                let mut vars: HashMap<&str, &str> = HashMap::new();
                let user_name = entity.get_full_name_with_id_and_email();
                vars.insert("user_name", &user_name);

                (
                    translator_service.variables(lang, "page.users.edit.title", &vars),
                    translator_service.variables(lang, "page.users.edit.header", &vars),
                    get_edit_url(entity.id.to_string().as_str()),
                )
            } else {
                (
                    translator_service.translate(lang, "page.users.create.title"),
                    translator_service.translate(lang, "page.users.create.header"),
                    get_create_url(),
                )
            };
            data
        }
        InvokeRoute::ProfileEdit | InvokeRoute::ProfileUpdate => {
            let title = translator_service.translate(lang, "page.profile.title");
            let heading = translator_service.translate(lang, "page.profile.header");
            let action = get_profile_url();
            (title, heading, action)
        }
    };

    context_data.title = title;

    //
    let mut is_done = false;

    if is_post {
        let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

        let executed =
            rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

        if !executed {
            let ttl_message = rate_limit_service.ttl_message_throw_http(
                translator_service,
                lang,
                &rate_limit_key,
            )?;
            errors.form.push(ttl_message)
        } else if errors.is_empty() {
            let id = if let Some(entity) = &entity {
                entity.id
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
            // user_data.avatar_id = data.avatar_id.to_owned();

            let mut columns: Vec<UserColumn> = vec![
                UserColumn::Email,
                UserColumn::Locale,
                UserColumn::Surname,
                UserColumn::Name,
                UserColumn::Patronymic,
                // UserColumn::AvatarId,
            ];

            if UserPolicy::can_set_roles(&auth_user, &user_roles) {
                user_data.roles_ids = data.roles_ids.to_owned();
                columns.push(UserColumn::RolesIds);
            }

            let columns: Option<Vec<UserColumn>> = Some(columns);

            let result = user_service.upsert(user_data.to_owned(), &columns);

            if let Err(error) = result {
                if error.eq(&UserServiceError::DuplicateEmail) {
                    errors.email.push(error.translate(lang, translator_service));
                } else {
                    errors.form.push(error.translate(lang, translator_service));
                }
            }

            if let Some(password) = &data.password {
                let result = if let Some(entity) = &entity {
                    user_service.update_password_by_id(entity.id, password)
                } else {
                    let email = data.email.to_owned().unwrap();
                    user_service.update_password_by_email(&email, password)
                };

                if let Err(error) = result {
                    if error.eq(&UserServiceError::PasswordHashFail) {
                        errors
                            .password
                            .push(error.translate(lang, translator_service));
                    } else {
                        errors.form.push(error.translate(lang, translator_service));
                    }
                }
            }

            is_done = errors.is_empty();
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

        if let Some(entity) = &entity {
            let user = user_service.first_by_id_throw_http(entity.id)?;
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
                let url_ = if is_profile {
                    "/"
                } else {
                    "/users"
                };

                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((LOCATION, HeaderValue::from_static(url_)))
                    .finish());
            }
        }
    }

    for variant in &alert_variants {
        context_data
            .alerts
            .push(Alert::from_variant(translator_service, lang, variant));
    }

    let layout_ctx = get_template_context(&context_data);

    let mut field_roles_ids: Option<Value> = None;
    if UserPolicy::can_set_roles(&auth_user, &user_roles) {
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

    let mut avatar_src: Option<String> = None;

    if let Some(entity) = &entity {
        if let Some(avatar_id) = &entity.avatar_id {
            if let Ok(user_file) = user_file_service.first_by_id(*avatar_id) {
                if let Some(user_file) = user_file {
                    avatar_src = user_file_service.get_public_path(&user_file);
                }
            }
        }
    }

    let fields = json!({
        "email": { "label": email_str, "value": &data.email, "errors": errors.email },
        "password": { "label": password_str, "value": &data.password, "errors": errors.password },
        "confirm_password": { "label": confirm_password_str, "value": &data.confirm_password, "errors": errors.confirm_password },
        "surname": { "label": surname_str, "value": &data.surname, "errors": errors.surname },
        "name": { "label": name_str, "value": &data.name, "errors": errors.name },
        "patronymic": { "label": patronymic_str, "value": &data.patronymic, "errors": errors.patronymic },
        "locale": { "label": locale_str, "value": &data.locale, "errors": errors.locale, "options": locales_, "placeholder": translator_service.translate(lang, "Not selected..."), },
        "roles_ids": field_roles_ids,
        "avatar": { "label": avatar_str, "errors": errors.avatar, "src": avatar_src },
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
    pub fn fill_from_user(&mut self, user: &User) {
        self.email = Some(user.email.to_owned());
        self.locale = user.locale.to_owned();
        self.surname = user.surname.to_owned();
        self.name = user.name.to_owned();
        self.patronymic = user.patronymic.to_owned();
        self.roles_ids = user.roles_ids.to_owned();
    }
    pub async fn prepare_from_multipart(
        &mut self,
        mut payload: Multipart,
        mut errors: ErrorMessages,
        entity: &Option<User>,
        lang: &str,
        translator_service: &TranslatorService,
        email_str: &str,
        password_str: &str,
        confirm_password_str: &str,
        locale_str: &str,
        surname_str: &str,
        name_str: &str,
        patronymic_str: &str,
        roles_ids_str: &str,
        avatar_str: &str,
        str_locales: &Vec<&str>,
    ) -> Result<ErrorMessages, Error> {
        // 2) Validate

        let mut roles_ids: Vec<u64> = Vec::new();
        while let Ok(Some(mut field)) = payload.try_next().await {
            let content_disposition = field.content_disposition();
            if content_disposition.is_none() {
                return Err(error::ErrorBadRequest(""));
            }
            let content_disposition = content_disposition.unwrap();

            let field_name = content_disposition.get_name();
            if field_name.is_none() {
                return Err(error::ErrorBadRequest(""));
            }
            let field_name = field_name.unwrap().to_string();

            let mime = field.content_type().cloned();
            let filename = content_disposition.get_filename();
            let filename = if let Some(filename) = filename {
                Some(filename.to_string())
            } else {
                None
            };

            let mut bytes = BytesMut::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|_| error::ErrorBadRequest(""))?;
                bytes.extend_from_slice(&data);
                match field_name.as_str() {
                    "_token" => {
                        if !BytesMutMaxLength::apply(&bytes, 400) {
                            bytes.clear();
                            break;
                        }
                    }
                    "avatar" => {
                        let mut errors_: Vec<String> = BytesMutMaxLength::validate(
                            translator_service,
                            lang,
                            &bytes,
                            USER_AVATAR_MAX_SIZE,
                            avatar_str
                        );
                        if !errors_.is_empty() {
                            errors.avatar.append(&mut errors_);
                        }

                        let mut errors_: Vec<String> = Mimes::validate(
                            translator_service,
                            lang,
                            &mime,
                            USER_AVATAR_MIMES,
                            avatar_str
                        );
                        if !errors_.is_empty() {
                            errors.avatar.append(&mut errors_);
                        }

                        if !errors.avatar.is_empty() {
                            bytes.clear();
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let bytes = bytes.freeze();

            match field_name.as_str() {
                "_token" => assign_value_bytes_to_string!(bytes, self._token),
                "action" => assign_value_bytes_to_string!(bytes, self.action),
                "email" => assign_value_bytes_to_string!(bytes, self.email),
                "password" => assign_value_bytes_to_string!(bytes, self.password),
                "confirm_password" => {
                    assign_value_bytes_to_string!(bytes, self.confirm_password)
                }
                "locale" => assign_value_bytes_to_string!(bytes, self.locale),
                "surname" => assign_value_bytes_to_string!(bytes, self.surname),
                "name" => assign_value_bytes_to_string!(bytes, self.name),
                "patronymic" => assign_value_bytes_to_string!(bytes, self.patronymic),
                "roles_ids[]" => {
                    let mut result: Option<String> = None;
                    assign_value_bytes_to_string!(bytes, result);
                    if let Some(result) = result {
                        let value = result.parse::<u64>();
                        if let Ok(value) = value {
                            roles_ids.push(value);
                        } else {
                            let mut vars = HashMap::new();
                            vars.insert("attribute", roles_ids_str);
                            errors.roles_ids.push(translator_service.variables(
                                lang,
                                "validation.integer",
                                &vars,
                            ));
                        }
                    }
                }
                "avatar" => {
                    if bytes.is_empty() {
                        self.avatar = None;
                    } else {
                        self.avatar = Some(Avatar {
                            bytes,
                            filename,
                            mime,
                        });
                    }
                }
                _ => {}
            };
        }

        if roles_ids.is_empty() {
            self.roles_ids = None;
        } else {
            self.roles_ids = Some(roles_ids);
        }

        // Validation action
        if let Some(value) = &self.action {
            let mut errors_: Vec<String> = ContainsStr::validate(
                translator_service,
                lang,
                value.as_str(),
                &Action::VARIANTS,
                "Action",
            );
            errors.form.append(&mut errors_);
        }

        // Validation email
        let mut errors_: Vec<String> = Required::validated(
            translator_service,
            lang,
            &self.email,
            |value| Email::validate(translator_service, lang, value, email_str),
            email_str,
        );
        errors.email.append(&mut errors_);

        // Validation password
        if entity.is_none() {
            let mut errors_: Vec<String> = Required::validated(
                translator_service,
                lang,
                &self.password,
                |value| MMCC::validate(translator_service, lang, value, 4, 255, password_str),
                password_str,
            );
            errors.password.append(&mut errors_);
        } else {
            if let Some(password) = &self.password {
                let mut errors_: Vec<String> =
                    MMCC::validate(translator_service, lang, password, 4, 255, password_str);
                errors.password.append(&mut errors_);
            }
        }

        // Validation confirm_password
        if entity.is_none() || self.password.is_some() {
            let mut errors_: Vec<String> = Required::validated(
                translator_service,
                lang,
                &self.confirm_password,
                |value| {
                    MMCC::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        confirm_password_str,
                    )
                },
                confirm_password_str,
            );
            errors.confirm_password.append(&mut errors_);
        }

        if errors.password.is_empty()
            && errors.confirm_password.is_empty()
            && self.password.is_some()
            && self.confirm_password.is_some()
        {
            let mut errors_: Vec<String> = Confirmed::validate(
                translator_service,
                lang,
                self.password.as_ref().unwrap(),
                self.confirm_password.as_ref().unwrap(),
                password_str,
            );
            errors.confirm_password.append(&mut errors_);
        }

        // Validation locale
        if let Some(locale) = &self.locale {
            let mut errors_: Vec<String> =
                ContainsVecStr::validate(translator_service, lang, locale, str_locales, locale_str);
            errors.locale.append(&mut errors_);
        }

        // Validation surname
        if let Some(surname) = &self.surname {
            errors.surname =
                StrMaxCharsCount::validate(translator_service, lang, surname, 255, surname_str);
        }

        // Validation name
        if let Some(name) = &self.name {
            errors.name = StrMaxCharsCount::validate(translator_service, lang, name, 255, name_str);
        }

        // Validation patronymic
        if let Some(patronymic) = &self.patronymic {
            errors.patronymic = StrMaxCharsCount::validate(
                translator_service,
                lang,
                patronymic,
                255,
                patronymic_str,
            );
        }

        Ok(errors)
    }
}
