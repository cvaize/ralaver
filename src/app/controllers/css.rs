use actix_web::{ Error, HttpResponse, Result };

pub async fn app() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css")
        .insert_header(("content-encoding", "gzip"))
        .body(RESOURCES_BUILD_APP_CSS_GZ))
}

static RESOURCES_BUILD_APP_CSS_GZ: &'static [u8] = include_bytes!("../../../resources/build/app.min.css.gz");