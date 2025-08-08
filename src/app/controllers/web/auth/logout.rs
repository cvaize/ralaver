use crate::app::middlewares::web_auth::REDIRECT_TO;
use crate::{AlertVariant, WebAuthService, WebHttpResponse};
use actix_web::http::header::HeaderValue;
use actix_web::web::Data;
use actix_web::{
    error, http::header::LOCATION, Error, HttpRequest, HttpResponse, Responder, Result,
};

pub async fn invoke(
    req: HttpRequest,
    web_auth_service: Data<WebAuthService>,
) -> Result<impl Responder, Error> {
    let web_auth_service = web_auth_service.get_ref();

    web_auth_service.logout_by_req(&req).map_err(|e| {
        log::error!("Logout:invoke - {e}");
        return error::ErrorInternalServerError("");
    })?;

    Ok(HttpResponse::SeeOther()
        .cookie(web_auth_service.make_clear_cookie())
        .set_alerts(vec![AlertVariant::LogoutSuccess])
        .insert_header((LOCATION, HeaderValue::from_static(REDIRECT_TO)))
        .finish())
}
