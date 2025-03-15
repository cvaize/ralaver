use crate::adapters::garde::GardeReportAdapter;
use actix_session::Session;
use actix_web::web::Redirect;
use actix_web::{error, web, Error, HttpResponse, Responder, Result};
use garde::Validate;
use handlebars::Handlebars;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Validate, Deserialize, Debug)]
pub struct SignInData {
    #[garde(required, inner(length(min = 1, max = 255)))]
    username: Option<String>,
    #[garde(required, inner(length(min = 1, max = 255)))]
    password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlashData {
    error: Option<String>,
    errors: Option<HashMap<String, String>>,
}

impl FlashData {
    pub fn empty() -> Self {
        Self {
            error: None,
            errors: None,
        }
    }
}

static FLASH_DATA_KEY: &str = "app.sign_in.flash_data";

pub async fn show(
    session: Session,
    tmpl: web::Data<Handlebars<'_>>,
) -> Result<impl Responder, Error> {
    // let mut tmpl: Handlebars = template::make();

    let flash_data = session
        .get::<String>(FLASH_DATA_KEY)
        .unwrap_or(None)
        .unwrap_or("{}".to_string());
    let flash_data: FlashData = serde_json::from_str(&flash_data).unwrap_or(FlashData::empty());

    let ctx = json!({
        "title": "Вход",
        "error" : &flash_data.error,
        "errors": &flash_data.errors
    });

    let s = tmpl
        .render("pages/auth/login.hbs", &ctx)
        .map_err(|e| error::ErrorInternalServerError("Template error"))?;
    session.remove(FLASH_DATA_KEY);
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    session: Session,
    data: web::Form<SignInData>,
) -> Result<impl Responder, Error> {
    let ctx;
    if let Err(report) = data.deref().validate() {
        let report_adapter = GardeReportAdapter::new(&report);
        let errors = report_adapter.to_hash_map();

        ctx = FlashData {
            error: Some("Ошибка валидации".to_string()),
            errors: Some(errors),
        };
    } else {
        ctx = FlashData::empty();
        // TODO: Auth
    };
    let json = serde_json::to_string(&ctx)?;
    session.insert(FLASH_DATA_KEY, json)?;

    Ok(Redirect::to("/login").see_other())
}
