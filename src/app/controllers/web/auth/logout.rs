use crate::{log_map_err, Session};
use crate::{AlertVariant, AuthService, WebHttpResponse};
use actix_web::web::Data;
use actix_web::{error, Error, HttpResponse, Responder, Result};

pub async fn invoke(
    session: Session,
    auth_service: Data<AuthService<'_>>,
) -> Result<impl Responder, Error> {
    let auth_service = auth_service.get_ref();

    auth_service
        .logout_from_session(&session)
        .map_err(log_map_err!(
            error::ErrorInternalServerError("AuthService error"),
            "Logout:invoke"
        ))?;

    Ok(HttpResponse::SeeOther()
        .set_alerts(vec![AlertVariant::LogoutSuccess])
        .insert_header((
            http::header::LOCATION,
            http::HeaderValue::from_static("/login"),
        ))
        .finish())
}
