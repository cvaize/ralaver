use crate::app::models::user::{PublicUser, User};
use crate::db_connection::DbPool;
use actix_web::{error, web, Error, HttpResponse, Result};
use serde_json::Value::Null;
use serde_json::{json, Value};
use std::collections::HashMap;
use actix_session::Session;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use tinytemplate::TinyTemplate;
use crate::schema::users as users_schema;

pub async fn index(
    session: Session,
    db_pool: web::Data<DbPool>,
    tmpl: web::Data<TinyTemplate<'_>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    // TODO: https://github.com/actix/actix-extras/blob/master/actix-session/examples/authentication.rs
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        // modify the session state
        session.insert("counter", count + 1)?;
    } else {
        session.insert("counter", 1)?;
    }

    let mut connection = db_pool.get().unwrap();

    let results: Vec<User> = users_schema::dsl::users
        .select(User::as_select())
        .limit(1)
        .load::<User>(&mut connection)
        .expect("Users load failed.");

    let result: Option<&User> = results.get(0);

    let user: Option<PublicUser> = match result {
        Some(user) => Some(user.to_public_user()),
        _ => None
    };

    let user: Value = serde_json::to_value(&user).unwrap_or(Null);

    let s = if let Some(name) = query.get("name") {
        // submitted form
        let ctx = json!({
          "name" : name.to_owned(),
          "text" : "Welcome!".to_owned(),
          "user" : user
        });
        tmpl.render("pages.home.user", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        let ctx = json!({
          "user" : user
        });
        // tmpl.render("pages.home.index", &serde_json::Value::Null)
        tmpl.render("pages.home.index", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
