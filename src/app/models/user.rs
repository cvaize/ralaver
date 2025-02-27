use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: Option<String>,
}
