use crate::app::middlewares::web_auth::REDIRECT_TO;
use crate::{AlertVariant, WebAuthService, WebHttpResponse};
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use crate::app::controllers::WEB_AUTH_SERVICE_ERROR;

pub async fn invoke(
    req: HttpRequest,
    web_auth_service: Data<WebAuthService>,
) -> Result<impl Responder, Error> {
    let web_auth_service = web_auth_service.get_ref();

    web_auth_service.logout_by_req(&req).map_err(|e| {
        log::error!("Logout:invoke - {e}");
        return error::ErrorInternalServerError(WEB_AUTH_SERVICE_ERROR);
    })?;

    Ok(HttpResponse::SeeOther()
        .cookie(web_auth_service.make_clear_cookie())
        .set_alerts(vec![AlertVariant::LogoutSuccess])
        .insert_header((
            http::header::LOCATION,
            http::HeaderValue::from_static(REDIRECT_TO),
        ))
        .finish())
}
