use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString as MMLS};
use crate::app::validator::rules::required::Required;
use crate::{prepare_value, Alert, AlertVariant, AppService, Locale, LocaleService, Permission, RateLimitService, Role, RoleColumn, RoleService, RoleServiceError, Session, TemplateService, TranslatableError, TranslatorService, User, UserColumn, WebAuthService, WebHttpResponse};
use actix_web::web::Path;
use actix_web::web::{Data, ReqData};
use crate::libs::actix_web::types::form::Form;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use strum::VariantNames;

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "roles_create_update";

const ROUTE_NAME: &'static str = "roles_create_update";

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
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    r_s: Data<RoleService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let data = Form(PostData::default());
    invoke(
        None, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, r_s, l_s,
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
    r_s: Data<RoleService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    invoke(
        None, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, r_s, l_s,
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
    r_s: Data<RoleService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let role_id = path.into_inner();
    let edit_role = r_s.get_ref().first_by_id_throw_http(role_id)?;
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
        edit_role, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, r_s, l_s,
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
    r_s: Data<RoleService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let role_id = path.into_inner();
    let edit_role = Some(r_s.get_ref().first_by_id_throw_http(role_id)?);
    invoke(
        edit_role, req, data, user, session, tr_s, tm_s, ap_s, wa_s, rl_s, r_s, l_s,
    )
}

pub fn invoke(
    edit_role: Option<Role>,
    req: HttpRequest,
    mut data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    r_s: Data<RoleService>,
    l_s: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let tr_s = tr_s.get_ref();
    let tm_s = tm_s.get_ref();
    let ap_s = ap_s.get_ref();
    let wa_s = wa_s.get_ref();
    let rl_s = rl_s.get_ref();
    let r_s = r_s.get_ref();
    let l_s = l_s.get_ref();

    //
    let user = user.as_ref();

    let mut alert_variants: Vec<AlertVariant> = Vec::new();
    let mut context_data = get_context_data(ROUTE_NAME, &req, user, &session, tr_s, ap_s, wa_s);

    let lang = &context_data.lang;

    let code_str = tr_s.translate(lang, "page.roles.create.fields.code");
    let name_str = tr_s.translate(lang, "page.roles.create.fields.name");
    let description_str = tr_s.translate(lang, "page.roles.create.fields.description");
    let permissions_str = tr_s.translate(lang, "page.roles.create.fields.permissions");

    let (title, heading, action) = if let Some(edit_role) = &edit_role {
        let mut vars: HashMap<&str, &str> = HashMap::new();
        let name_ = &edit_role.name;
        vars.insert("name", name_);

        (
            tr_s.variables(lang, "page.roles.edit.title", &vars),
            tr_s.variables(lang, "page.roles.edit.header", &vars),
            get_edit_url(edit_role.id.to_string().as_str()),
        )
    } else {
        (
            tr_s.translate(lang, "page.roles.create.title"),
            tr_s.translate(lang, "page.roles.create.header"),
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
            errors.code = Required::validated(tr_s, lang, &data.code, |value| {
                MMLS::validate(tr_s, lang, value, 4, 255, &code_str)
            }, &code_str);

            errors.name = Required::validated(tr_s, lang, &data.name, |value| {
                MMLS::validate(tr_s, lang, value, 4, 255, &name_str)
            }, &name_str);

            if let Some(description) = &data.description {
                errors.description = MaxLengthString::validate(tr_s, lang, description, 255, &description_str);
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

                let result = r_s.upsert(&mut role_data, &columns);

                if let Err(error) = result {
                    if error.eq(&RoleServiceError::DuplicateCode) {
                        errors.code.push(error.translate(lang, tr_s));
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

        if let Some(edit_role) = &edit_role {
            let user = r_s.first_by_id_throw_http(edit_role.id)?;
            id = user.id.to_string();
            let name_ = user.name;
            alert_variants.push(AlertVariant::RolesUpdateSuccess(name_))
        } else if let Some(code_) = &data.code {
            let user = r_s.first_by_code_throw_http(code_)?;
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
                        http::HeaderValue::from_static("/roles"),
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
            "label": tr_s.translate(lang, &key),
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
            "main": tr_s.translate(lang, "page.roles.create.tabs.main"),
            "permissions": tr_s.translate(lang, "page.roles.create.tabs.permissions"),
        },
        "breadcrumbs": [
            {"href": "/", "label": tr_s.translate(lang, "page.home.header")},
            {"href": "/roles", "label": tr_s.translate(lang, "page.roles.index.header")},
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
                "href": "/roles"
            },
        },
    });
    let s = tm_s.render_throw_http("pages/roles/create-update.hbs", &ctx)?;
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
