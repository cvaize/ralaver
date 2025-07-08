use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString as MMLS};
use crate::app::validator::rules::required::Required;
use crate::libs::actix_web::types::form::Form;
use crate::{
    prepare_value, Alert, AlertVariant, AppService, Permission, RateLimitService, Role, RoleColumn,
    RolePolicy, RoleService, RoleServiceError, Session, TemplateService, TranslatableError,
    TranslatorService, User, WebAuthService, WebHttpResponse,
};
use actix_web::{web::{Path, Data, ReqData}, error, Error, HttpRequest, HttpResponse, Result, http::{Method, header::{LOCATION}}};
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use actix_web::http::header::HeaderValue;
use strum::VariantNames;

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "roles_create_update";

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub action: Option<String>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub code: Vec<String>,
    pub name: Vec<String>,
    pub description: Vec<String>,
    pub permissions: Vec<String>,
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
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let roles = role_service.all_throw_http()?;
    if !RolePolicy::can_create(&user, &roles) {
        return Err(error::ErrorForbidden(""));
    }
    let data = Form(PostData::default());
    invoke(
        None,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
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
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let roles = role_service.all_throw_http()?;
    if !RolePolicy::can_create(&user, &roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
        None,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
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
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let roles = role_service.all_throw_http()?;
    if !RolePolicy::can_update(&user, &roles) {
        return Err(error::ErrorForbidden(""));
    }
    let role_id = path.into_inner();
    let edit_role = role_service.get_ref().first_by_id_throw_http(role_id)?;
    let post_data = PostData {
        _token: None,
        action: None,
        code: Some(edit_role.code.to_owned()),
        name: Some(edit_role.name.to_owned()),
        description: edit_role.description.to_owned(),
        permissions: edit_role.permissions.to_owned(),
    };
    let edit_role = Some(edit_role);
    let data = Form(post_data);
    invoke(
        edit_role,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
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
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let roles = role_service.all_throw_http()?;
    if !RolePolicy::can_update(&user, &roles) {
        return Err(error::ErrorForbidden(""));
    }
    let role_id = path.into_inner();
    let edit_role = Some(role_service.get_ref().first_by_id_throw_http(role_id)?);
    invoke(
        edit_role,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
    )
}

pub fn invoke(
    edit_role: Option<Role>,
    req: HttpRequest,
    mut data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
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

    let code_str = translator_service.translate(lang, "page.roles.create.fields.code");
    let name_str = translator_service.translate(lang, "page.roles.create.fields.name");
    let description_str = translator_service.translate(lang, "page.roles.create.fields.description");
    let permissions_str = translator_service.translate(lang, "page.roles.create.fields.permissions");

    let (title, heading, action) = if let Some(edit_role) = &edit_role {
        let mut vars: HashMap<&str, &str> = HashMap::new();
        let name_ = &edit_role.name;
        vars.insert("name", name_);

        (
            translator_service.variables(lang, "page.roles.edit.title", &vars),
            translator_service.variables(lang, "page.roles.edit.header", &vars),
            get_edit_url(edit_role.id.to_string().as_str()),
        )
    } else {
        (
            translator_service.translate(lang, "page.roles.create.title"),
            translator_service.translate(lang, "page.roles.create.header"),
            get_create_url(),
        )
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
            errors.code = Required::validated(
                translator_service,
                lang,
                &data.code,
                |value| MMLS::validate(translator_service, lang, value, 4, 255, &code_str),
                &code_str,
            );

            errors.name = Required::validated(
                translator_service,
                lang,
                &data.name,
                |value| MMLS::validate(translator_service, lang, value, 4, 255, &name_str),
                &name_str,
            );

            if let Some(description) = &data.description {
                errors.description =
                    MaxLengthString::validate(translator_service, lang, description, 255, &description_str);
            }

            if errors.is_empty() {
                let id = if let Some(edit_role) = &edit_role {
                    edit_role.id
                } else {
                    0
                };
                let mut role_data = Role::default();
                role_data.id = id;
                role_data.code = data.code.clone().unwrap();
                role_data.name = data.name.clone().unwrap();
                role_data.description = data.description.to_owned();
                role_data.permissions = data.permissions.to_owned();

                let columns: Option<Vec<RoleColumn>> = Some(vec![
                    RoleColumn::Code,
                    RoleColumn::Name,
                    RoleColumn::Description,
                    RoleColumn::Permissions,
                ]);

                let result = role_service.upsert(role_data, &columns);

                if let Err(error) = result {
                    if error.eq(&RoleServiceError::DuplicateCode) {
                        errors.code.push(error.translate(lang, translator_service));
                    } else {
                        errors.form.push(error.translate(lang, translator_service));
                    }
                } else {
                    is_done = true;
                }
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

        if let Some(edit_role) = &edit_role {
            let user = role_service.first_by_id_throw_http(edit_role.id)?;
            id = user.id.to_string();
            let name_ = user.name;
            alert_variants.push(AlertVariant::RolesUpdateSuccess(name_))
        } else if let Some(code_) = &data.code {
            let user = role_service.first_by_code_throw_http(code_)?;
            id = user.id.to_string();
            let name_ = user.name;
            alert_variants.push(AlertVariant::RolesCreateSuccess(name_))
        }

        if let Some(action) = &data.action {
            if action.eq("save") {
                let url_ = get_edit_url(&id);
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
                        HeaderValue::from_static("/roles"),
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

    let layout_ctx = get_template_context(&context_data);
    let mut permissions: Vec<Value> = Vec::new();

    for variant in Permission::VARIANTS {
        let mut key = "permission.".to_string();
        key.push_str(variant);
        let mut checked = false;
        if let Some(val) = &data.permissions {
            let variant_ = variant.to_string();
            if val.contains(&variant_) {
                checked = true;
            }
        }
        permissions.push(json!({
            "label": translator_service.translate(lang, &key),
            "value": variant,
            "checked": checked
        }));
    }

    let fields = json!({
        "code": { "label": code_str, "value": &data.code, "errors": errors.code },
        "name": { "label": name_str, "value": &data.name, "errors": errors.name },
        "description": { "label": description_str, "value": &data.description, "errors": errors.description },
        "permissions": { "label": permissions_str, "value": &data.permissions, "errors": errors.permissions, "options": permissions },
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "tabs": {
            "main": translator_service.translate(lang, "page.roles.create.tabs.main"),
            "permissions": translator_service.translate(lang, "page.roles.create.tabs.permissions"),
        },
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/roles", "label": translator_service.translate(lang, "page.roles.index.header")},
            {"label": &heading},
        ],
        "form": {
            "action": &action,
            "method": "post",
            "fields": fields,
            "save": translator_service.translate(lang, "Save"),
            "save_and_close": translator_service.translate(lang, "Save and close"),
            "close": {
                "label": translator_service.translate(lang, "Close"),
                "href": "/roles"
            },
        },
    });
    let s = template_service.render_throw_http("pages/roles/create-update.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

pub fn get_create_url() -> String {
    "/roles/create".to_string()
}

pub fn get_edit_url(id: &str) -> String {
    let mut str_ = "/roles/".to_string();
    str_.push_str(id);
    str_
}

impl PostData {
    pub fn prepare(&mut self) {
        prepare_value!(self._token);
        prepare_value!(self.action);
        prepare_value!(self.code);
        prepare_value!(self.name);
        prepare_value!(self.description);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.len() == 0
            && self.code.len() == 0
            && self.name.len() == 0
            && self.description.len() == 0
    }
}
