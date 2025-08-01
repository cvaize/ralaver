use crate::helpers::join_vec;
use crate::libs::actix_web::types::form::Form;
use crate::{
    AlertVariant, FilePolicy, FileService, LocaleService, RateLimitService, RoleService, Session,
    TranslatorService, User, WebAuthService, WebHttpResponse,
};
use actix_web::{
    error,
    http::header::{HeaderValue, LOCATION, ORIGIN, REFERER},
    web::{Data, ReqData},
    Error, HttpRequest, HttpResponse, Result,
};
use serde_derive::Deserialize;
use std::sync::Arc;

const RL_MAX_ATTEMPTS: u64 = 30;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "files_mass_actions";

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub selected: Option<Vec<u64>>,
    pub action: Option<String>,
}

pub async fn invoke(
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    file_service: Data<FileService>,
    role_service: Data<RoleService>,
    locale_service: Data<LocaleService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let locale_service = locale_service.get_ref();
    let role_service = role_service.get_ref();
    let file_service = file_service.get_ref();
    let translator_service = translator_service.get_ref();

    web_auth_service.check_csrf_throw_http(&session, &data._token)?;

    let user_roles = role_service.all_throw_http()?;

    let user = user.as_ref();
    let lang: String = locale_service.get_locale_code(Some(&req), Some(&user));

    let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

    let mut alert_variants = Vec::new();
    let executed =
        rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

    if executed {
        if data.action.is_some() && data.selected.is_some() {
            let action = data.action.as_ref().unwrap();
            let ids = data.selected.as_ref().unwrap();
            if ids.len() > 0 {
                if action.eq("delete") {
                    if !FilePolicy::can_delete(&user, &user_roles) {
                        return Err(error::ErrorForbidden(""));
                    }
                    file_service.soft_delete_by_ids_throw_http(ids)?;
                    alert_variants.push(AlertVariant::FilesMassDeleteSuccess(join_vec(ids, ", ")));
                } else if action.eq("restore") {
                    if !FilePolicy::can_delete(&user, &user_roles) {
                        return Err(error::ErrorForbidden(""));
                    }
                    file_service.restore_by_ids_throw_http(ids)?;
                    alert_variants.push(AlertVariant::FilesMassRestoreSuccess(join_vec(ids, ", ")));
                }
            }
        }
    } else {
        let alert_variant = rate_limit_service.alert_variant_throw_http(
            translator_service,
            &lang,
            &rate_limit_key,
        )?;
        alert_variants.push(alert_variant);
    }

    let headers = req.headers();
    let default = HeaderValue::from_static("/files");
    let location = headers
        .get(REFERER)
        .unwrap_or(headers.get(ORIGIN).unwrap_or(&default));
    let location = location.to_str().unwrap_or("/files");

    Ok(HttpResponse::SeeOther()
        .set_alerts(alert_variants)
        .insert_header((LOCATION, HeaderValue::from_str(location).unwrap_or(default)))
        .finish())
}
