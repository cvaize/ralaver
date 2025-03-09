use std::ops::Deref;
// use std::collections::HashMap;
// use actix_session::Session;
use actix_web::{error, web, Error, HttpResponse, Result};
use garde::Validate;
use serde_json::json;
use serde_json::Value::Null;
use tinytemplate::TinyTemplate;
use serde_derive::Deserialize;
// use crate::db_connection::DbPool;

#[derive(Validate, Deserialize, Debug)]
pub struct SignInData {
    #[garde(required, inner(length(min = 1, max = 255)))]
    username: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    password: Option<String>,
}

pub async fn show(tmpl: web::Data<TinyTemplate<'_>>) -> Result<HttpResponse, Error> {
    let ctx = json!({
       "error" : &Null,
    });
    let s = tmpl
        .render("pages.auth.login", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    tmpl: web::Data<TinyTemplate<'_>>,
    data: web::Form<SignInData>,
) -> Result<HttpResponse, Error> {

    let s = if let Err(e) = data.deref().validate() {
        dbg!(e);
        // submitted form
        let ctx = json!({
          "error" : "Ошибка".to_owned(),
        });
        tmpl.render("pages.auth.login", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        let ctx = json!({
           "error" : &Null,
        });
        tmpl.render("pages.auth.login", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
