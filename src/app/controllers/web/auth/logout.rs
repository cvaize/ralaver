use crate::AuthService;
use crate::{log_map_err, FlashService, Session};
use actix_web::web::Data;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, Responder, Result};

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

    Ok(Redirect::to("/login").see_other())
}
