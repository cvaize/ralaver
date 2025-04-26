use crate::RateLimitService;
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};

pub async fn index(
    req: HttpRequest,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    let rate_limit_service = rate_limit_service.get_ref();
    let key = rate_limit_service
        .make_key_from_request(&req)
        .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(key))
}
