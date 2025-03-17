use crate::adapters::garde::GardeReportAdapter;
use crate::app::services::auth::{Auth, Credentials};
use crate::db_connection::DbPool;
use actix_session::Session;
use actix_web::web::Redirect;
use actix_web::{error, web, Error, HttpResponse, Responder, Result};
use garde::Validate;
use handlebars::Handlebars;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug)]
struct FlashData {
    success: Option<String>,
    error: Option<String>,
    errors: Option<HashMap<String, String>>,
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
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    session.remove(FLASH_DATA_KEY);
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn sign_in(
    session: Session,
    data: web::Form<Credentials>,
    db_pool: web::Data<DbPool>,
) -> Result<impl Responder, Error> {
    let ctx;
    let mut is_redirect_login = true;
    let credentials = data.deref();
    if let Err(report) = credentials.validate() {
        let report_adapter = GardeReportAdapter::new(&report);
        let errors = report_adapter.to_hash_map();

        ctx = FlashData::errors(Some(errors));
    } else {
        let auth_result = Auth::authenticate(&db_pool, credentials);

        ctx = match auth_result {
            Ok(user_id) => match Auth::insert_user_id_into_session(&session, user_id) {
                Ok(()) => {
                    is_redirect_login = false;
                    FlashData::success(Some("Авторизация успешно пройдена.".to_string()))
                }
                _ => FlashData::error(Some("Авторизация не пройдена.".to_string())),
            },
            _ => FlashData::error(Some("Авторизация не пройдена.".to_string())),
        };
    };
    let json = serde_json::to_string(&ctx)?;
    session
        .insert(FLASH_DATA_KEY, json)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    if is_redirect_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        Ok(Redirect::to("/").see_other())
    }
}

impl FlashData {
    pub fn empty() -> Self {
        Self {
            success: None,
            error: None,
            errors: None,
        }
    }
    pub fn success(success: Option<String>) -> Self {
        Self {
            success,
            error: None,
            errors: None,
        }
    }
    pub fn error(error: Option<String>) -> Self {
        Self {
            success: None,
            error,
            errors: None,
        }
    }
    pub fn errors(errors: Option<HashMap<String, String>>) -> Self {
        Self {
            success: None,
            error: None,
            errors,
        }
    }
}
