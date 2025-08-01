use crate::{
    AlertVariant, LocaleService, RateLimitService, RoleService, Session, TranslatorService, User,
    UserPolicy, UserService, WebAuthService, WebHttpResponse,
};
use actix_web::{web::{Data, Form, Path, ReqData}, error, Error, HttpRequest, HttpResponse, Result, http::header::{HeaderValue, ORIGIN, REFERER, LOCATION}};
use serde_derive::Deserialize;
use std::sync::Arc;

const RL_MAX_ATTEMPTS: u64 = 60;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "users_delete";

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
}

pub async fn invoke(
    req: HttpRequest,
    path: Path<u64>,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    translator_service: Data<TranslatorService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let locale_service = locale_service.get_ref();
    let user_service = user_service.get_ref();
    let translator_service = translator_service.get_ref();
    let role_service = role_service.get_ref();

    web_auth_service.check_csrf_throw_http(&session, &data._token)?;

    let roles = role_service.all_throw_http()?;
    if !UserPolicy::can_delete(&user, &roles) {
        return Err(error::ErrorForbidden(""));
    }

    let user_id = path.into_inner();
    let user = user.as_ref();
    let lang: String = locale_service.get_locale_code(Some(&req), Some(&user));
    let delete_user = user_service.first_by_id_throw_http(user_id)?;

    let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

    let mut alert_variants = Vec::new();
    let executed =
        rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

    if executed {
        user_service.delete_by_id_throw_http(delete_user.id)?;
        let name = delete_user.get_full_name_with_id_and_email();
        alert_variants.push(AlertVariant::UsersDeleteSuccess(name));
    } else {
        let alert_variant = rate_limit_service.alert_variant_throw_http(
            translator_service,
            &lang,
            &rate_limit_key,
        )?;
        alert_variants.push(alert_variant);
    }

    let headers = req.headers();
    let default = HeaderValue::from_static("/users");
    let location = headers
        .get(REFERER)
        .unwrap_or(headers.get(ORIGIN).unwrap_or(&default));
    let location = location.to_str().unwrap_or("/users");

    Ok(HttpResponse::SeeOther()
        .set_alerts(alert_variants)
        .insert_header((
            LOCATION,
            HeaderValue::from_str(location).unwrap_or(default),
        ))
        .finish())
}
