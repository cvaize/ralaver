use actix_web::{ Error, HttpResponse, Result };

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("application/json").body("{\"test\": 1}"))
}