use crate::{AlertVariant, AuthService, WebHttpResponse};
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};

pub async fn invoke(
    req: HttpRequest,
    auth_service: Data<AuthService<'_>>,
) -> Result<impl Responder, Error> {
    let auth_service = auth_service.get_ref();

    auth_service.logout_by_req(&req).map_err(|e| {
        log::error!("Logout:invoke - {e}");
        return error::ErrorInternalServerError("AuthService error");
    })?;

    Ok(HttpResponse::SeeOther()
        .cookie(auth_service.make_auth_token_clear_cookie())
        .set_alerts(vec![AlertVariant::LogoutSuccess])
        .insert_header((
            http::header::LOCATION,
            http::HeaderValue::from_static("/login"),
        ))
        .finish())
}
