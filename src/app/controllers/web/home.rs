use crate::app::models::user::User;
use crate::db_connection::DbPool;
use crate::schema::users::dsl::users;
use actix_web::{error, web, Error, HttpResponse, Result};
use diesel::RunQueryDsl;
use serde_json::json;
use std::collections::HashMap;
use tinytemplate::TinyTemplate;

pub async fn index(
    db_pool: web::Data<DbPool>,
    tmpl: web::Data<TinyTemplate<'_>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let mut connection = db_pool.get().unwrap();

    let results: Vec<User> = users
        .load::<User>(&mut connection)
        .expect("Users load failed.");
    let default_user = User {
        id: 0,
        username: "Null".to_string(),
        password: None,
    };
    let user = &results.get(0).unwrap_or(&default_user);

    let s = if let Some(name) = query.get("name") {
        // submitted form
        let ctx = json!({
          "name" : name.to_owned(),
          "text" : "Welcome!".to_owned(),
          "user" : {
                "id": &user.id,
                "username": &user.username,
            }
        });
        tmpl.render("pages.home.user", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        let ctx = json!({
          "user" : {
                "id": &user.id,
                "username": &user.username,
            }
        });
        // tmpl.render("pages.home.index", &serde_json::Value::Null)
        tmpl.render("pages.home.index", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
