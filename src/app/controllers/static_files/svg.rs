use actix_web::{ Error, HttpResponse, Result };

pub async fn logo() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type(mime::IMAGE_SVG.as_ref())
        .insert_header(("content-encoding", "gzip"))
        .body(RESOURCES_LOGO_SVG_GZ))
}

static RESOURCES_LOGO_SVG_GZ: &'static [u8] = include_bytes!("../../../../resources/dist/logo.svg.gz");