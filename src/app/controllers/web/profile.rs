use crate::app::controllers::web::users::create_update::{invoke as users_create_update_invoke, post_data_from_user, PostData};
use crate::libs::actix_web::types::form::Form;
use crate::{
    AppService, LocaleService, RateLimitService, RoleService, TemplateService, TranslatorService,
    UserService,
};
use crate::{Session, User, WebAuthService};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use std::sync::Arc;

pub const ROUTE_NAME: &'static str = "profile_index";

pub async fn index(
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
    r_s: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let user_roles = r_s.get_all_throw_http()?;
    let edit_user = user.as_ref().clone();
    let post_data = post_data_from_user(&edit_user);
    let edit_user = Some(edit_user);
    let data = Form(post_data);
    users_create_update_invoke(
        true, edit_user, req, data, user, user_roles, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s,
        l_s, r_s,
    )
}

pub async fn update(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    data: Form<PostData>,
    tr_s: Data<TranslatorService>,
    tm_s: Data<TemplateService>,
    ap_s: Data<AppService>,
    wa_s: Data<WebAuthService>,
    rl_s: Data<RateLimitService>,
    u_s: Data<UserService>,
    l_s: Data<LocaleService>,
    r_s: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let user_roles = r_s.get_all_throw_http()?;
    let edit_user = user.as_ref().clone();
    let edit_user = Some(edit_user);
    users_create_update_invoke(
        true, edit_user, req, data, user, user_roles, session, tr_s, tm_s, ap_s, wa_s, rl_s, u_s,
        l_s, r_s,
    )
}

pub fn get_url() -> String {
    "/profile".to_string()
}
