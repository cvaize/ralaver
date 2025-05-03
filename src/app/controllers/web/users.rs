use std::sync::Arc;
use actix_web::web::{Data, Form, ReqData};
use actix_web::{Error, HttpResponse, Result};
use serde_derive::Deserialize;
use crate::{Session, WebAuthService};

#[derive(Deserialize, Debug)]
pub struct TestData {
    pub _token: Option<String>,
    pub test: Option<String>,
}

pub async fn index(
    data: Form<TestData>,
    session: ReqData<Arc<Session>>,
    web_auth_service: Data<WebAuthService>,
) -> Result<HttpResponse, Error> {
    let web_auth_service = web_auth_service.get_ref();
    let session = session.as_ref();

    web_auth_service.check_csrf_throw_http(session, &data._token)?;

    dbg!(&data._token);
    dbg!(&data.test);

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
