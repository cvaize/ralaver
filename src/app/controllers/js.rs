use actix_web::{ Error, HttpResponse, Result };

pub async fn app() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("application/javascript")
        .insert_header(("content-encoding", "gzip"))
        .body(RESOURCES_BUILD_APP_JS_GZ))
}

static RESOURCES_BUILD_APP_JS_GZ: &'static [u8] = include_bytes!("../../../resources/build/app.min.js.gz");