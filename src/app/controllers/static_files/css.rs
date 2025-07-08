use actix_web::{ Error, HttpResponse, Result };

pub async fn app() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type(mime::TEXT_CSS_UTF_8.as_ref())
        .insert_header(("content-encoding", "gzip"))
        .body(RESOURCES_BUILD_APP_CSS_GZ))
}

static RESOURCES_BUILD_APP_CSS_GZ: &'static [u8] = include_bytes!("../../../../resources/dist/app.min.css.gz");