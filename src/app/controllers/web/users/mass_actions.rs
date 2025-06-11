use crate::libs::actix_web::types::form::Form;
use crate::{
    AlertVariant, LocaleService, RateLimitService, RoleService, Session, TranslatorService, User,
    UserPolicy, UserService, WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::header::{ORIGIN, REFERER};
use http::HeaderValue;
use serde_derive::Deserialize;
use std::sync::Arc;

const RL_MAX_ATTEMPTS: u64 = 30;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "users_mass_actions";

#[derive(Deserialize, Default, Debug)]
pub struct MassActionsData {
    pub _token: Option<String>,
    pub selected: Option<Vec<u64>>,
    pub action: Option<String>,
}

pub async fn invoke(
    req: HttpRequest,
    data: Form<MassActionsData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    tr_s: Data<TranslatorService>,
    r_s: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let wa_s = wa_s.get_ref();
    let rl_s = rl_s.get_ref();
    let l_s = l_s.get_ref();
    let u_s = u_s.get_ref();
    let tr_s = tr_s.get_ref();
    let r_s = r_s.get_ref();

    wa_s.check_csrf_throw_http(&session, &data._token)?;

    let roles = r_s.get_all_throw_http()?;

    let user = user.as_ref();
    let lang: String = l_s.get_locale_code(Some(&req), Some(&user));

    let rate_limit_key = rl_s.make_key_from_request_throw_http(&req, RL_KEY)?;

    let mut alert_variants = Vec::new();
    let executed = rl_s.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

    if executed {
        if data.action.is_some() && data.selected.is_some() {
            let action = data.action.as_ref().unwrap();
            let ids = data.selected.as_ref().unwrap();
            if ids.len() > 0 {
                if action.eq("delete") {
                    if !UserPolicy::can_delete(&user, &roles) {
                        return Err(error::ErrorForbidden(""));
                    }
                    u_s.delete_by_ids_throw_http(ids)?;
                    alert_variants.push(AlertVariant::UsersMassDeleteSuccess(
                        ids.iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                    ));
                }
            }
        }
    } else {
        let alert_variant = rl_s.alert_variant_throw_http(tr_s, &lang, &rate_limit_key)?;
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
            http::header::LOCATION,
            HeaderValue::from_str(location).unwrap_or(default),
        ))
        .finish())
}
