use std::collections::HashMap;

use actix_web::{
    error,
    web, Error, HttpResponse, Result,
};
use serde_json::json;
use tinytemplate::TinyTemplate;

pub async fn index(
    tmpl: web::Data<TinyTemplate<'_>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let s = if let Some(name) = query.get("name") {
        // submitted form
        let ctx = json!({
          "name" : name.to_owned(),
          "text" : "Welcome!".to_owned()
        });
        tmpl.render("pages.home.user", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        tmpl.render("pages.home.index", &serde_json::Value::Null)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}