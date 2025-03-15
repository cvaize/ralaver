use std::ops::Deref;
// use std::collections::HashMap;
// use actix_session::Session;
use crate::adapters::garde::GardeReportAdapter;
use actix_web::{error, web, Error, HttpResponse, Result};
use garde::external::compact_str::ToCompactString;
use garde::{Report, Validate};
use handlebars::Handlebars;
use serde_derive::Deserialize;
use serde_json::json;
use serde_json::Value::Null;
use crate::app::providers::template;
// use crate::db_connection::DbPool;

#[derive(Validate, Deserialize, Debug)]
pub struct SignInData {
    #[garde(required, inner(length(min = 1, max = 255)))]
    username: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    password: Option<String>,
}

pub async fn show(tmpl: web::Data<Handlebars<'_>>) -> Result<HttpResponse, Error> {
    let ctx = json!({
       "error" : &Null,
    });
    let s = tmpl
        .render("pages/auth/login.hbs", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    tmpl: web::Data<Handlebars<'_>>,
    data: web::Form<SignInData>,
) -> Result<HttpResponse, Error> {
    // let mut tmpl: Handlebars = template::make();

    let s = if let Err(report) = data.deref().validate() {
        let report_adapter = GardeReportAdapter::new(&report);
        let errors = report_adapter.to_hash_map();
        let ctx = json!({
            // "error" : "Ошибка".to_owned(),
            "errors": errors
        });
        tmpl.render("pages/auth/login.hbs", &ctx)
            .map_err(|e| {
                dbg!(e);
                return error::ErrorInternalServerError("Template error")
            })?
    } else {
        let ctx = json!({
           "error" : &Null,
        });
        tmpl.render("pages/auth/login.hbs", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
