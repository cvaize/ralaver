use actix_web::{ Error, HttpResponse, Result };

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type(mime::APPLICATION_JSON.as_ref()).body("{\"test\": 1}"))
}