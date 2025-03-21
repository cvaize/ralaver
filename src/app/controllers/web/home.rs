use crate::app::models::user::User;
use crate::app::services::auth::{Auth};
use crate::app::services::session::{SessionFlashData, SessionFlashService};
use crate::db_connection::DbPool;
use actix_session::Session;
use actix_web::{error, web, Error, HttpResponse, Responder, Result};
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use handlebars::Handlebars;
use serde_json::Value::Null;
use serde_json::{json, Value};
use std::collections::HashMap;

pub async fn index(
    session: Session,
    db_pool: web::Data<DbPool>,
    tmpl: web::Data<Handlebars<'_>>,
    query: web::Query<HashMap<String, String>>,
    auth: Auth
) -> Result<impl Responder, Error> {
    let user = auth.authenticate_from_session()
        .map_err(|_| error::ErrorUnauthorized("Unauthorized"))?;

    dbg!(user);


    let flash_data: SessionFlashData =
        SessionFlashService::new(&session, None)
            .read_and_forget()
            .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    // TODO: https://github.com/actix/actix-extras/blob/master/actix-session/examples/authentication.rs
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        // modify the session state
        session.insert("counter", count + 1)?;
    } else {
        session.insert("counter", 1)?;
    }

    let mut connection = db_pool
        .get()
        .map_err(|_| error::ErrorInternalServerError("Db error"))?;

    let results: Vec<User> = crate::schema::users::dsl::users
        .select(User::as_select())
        .limit(1)
        .load::<User>(&mut connection)
        .expect("Users load failed.");

    let user: Option<&User> = results.get(0);

    let user: Value = serde_json::to_value(&user).unwrap_or(Null);

    let s = if let Some(name) = query.get("name") {
        // submitted form
        let ctx = json!({
            "name" : name.to_owned(),
            "text" : "Welcome!".to_owned(),
            "user" : user,
            "alerts": flash_data.alerts
        });
        // tmpl.render("pages/home/user.html", &ctx)
        //     .map_err(|_| error::ErrorInternalServerError("Template error"))?
        tmpl.render("pages/home/user.hbs", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        let ctx = json!({
            "user" : user,
            "alerts": flash_data.alerts
        });
        // tmpl.render("pages/home/index.html", &serde_json::Value::Null)
        tmpl.render("pages/home/index.hbs", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
