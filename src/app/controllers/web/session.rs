use actix_web::{error, Error, HttpResponse, Result};
use serde_derive::Deserialize;
use crate::app::services::session::SessionService;

#[derive(Deserialize, Debug)]
pub struct DarkModeData {
    pub dark_mode: Option<bool>
}

pub async fn dark_mode(
    data: actix_web::web::Json<DarkModeData>,
    session_service: SessionService
) -> Result<HttpResponse, Error> {
    let dark_mode = data.into_inner().dark_mode.unwrap_or(false);

    session_service.set_dark_mode(dark_mode)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(HttpResponse::Ok().content_type("application/json").body("{\"status\": \"ok\"}"))
}