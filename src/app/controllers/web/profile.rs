use crate::app::controllers::web::users::create_update::{
    invoke as users_create_update_invoke, InvokeData, InvokeRoute,
};
use crate::{
    AppService, FileService, LocaleService, RateLimitService, RoleService, TemplateService,
    TranslatorService, UserFileService, UserService,
};
use crate::{Session, User, WebAuthService};
use actix_multipart::Multipart;
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use std::sync::Arc;

pub async fn index(
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
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    users_create_update_invoke(InvokeData {
        route: InvokeRoute::ProfileEdit,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: None,
        payload: None,
        req,
        // Services
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
        file_service,
    })
    .await
}

pub async fn update(
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
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    users_create_update_invoke(InvokeData {
        route: InvokeRoute::ProfileUpdate,
        auth_user: user.as_ref(),
        auth_session: session.as_ref(),
        entity: None,
        payload: Some(payload),
        req,
        // Services
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
        role_service,
        user_file_service,
        file_service,
    })
    .await
}

pub fn get_url() -> String {
    "/profile".to_string()
}
