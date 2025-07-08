use crate::app::controllers::web::users::create_update::{
    invoke as users_create_update_invoke, post_data_from_user, PostData,
};
use crate::libs::actix_web::types::form::Form;
use crate::{
    AppService, LocaleService, RateLimitService, RoleService, TemplateService, TranslatorService,
    UserFileService, UserService,
};
use crate::{Session, User, WebAuthService};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use std::sync::Arc;

pub async fn index(
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
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    let edit_user = user.as_ref().clone();
    let post_data = post_data_from_user(&edit_user);
    let edit_user = Some(edit_user);
    let data = Form(post_data);
    users_create_update_invoke(
        true,
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
        user_file_service,
    )
}

pub async fn update(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    data: Form<PostData>,
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
    let user_roles = role_service.get_all_throw_http()?;
    let edit_user = user.as_ref().clone();
    let edit_user = Some(edit_user);
    users_create_update_invoke(
        true,
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
        user_file_service,
    )
}

pub fn get_url() -> String {
    "/profile".to_string()
}
