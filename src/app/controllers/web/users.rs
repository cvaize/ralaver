use actix_web::web::Form;
use actix_web::{Error, HttpResponse, Result};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TestData {
    pub csrf: Option<String>,
    pub test: Option<String>,
}

pub async fn index(
    data: Form<TestData>,
) -> Result<HttpResponse, Error> {
    dbg!(data);

    Ok(HttpResponse::SeeOther()
        .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
        .finish())

    // let rate_limit_service = rate_limit_service.get_ref();
    // let key = rate_limit_service
    //     .make_key_from_request(&req)
    //     .map_err(|_| error::ErrorInternalServerError("RateLimitService error"))?;
    //
    // Ok(HttpResponse::Ok().content_type("text/html").body(key))
}
