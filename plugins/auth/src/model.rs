use crate::schema;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(Sqlite))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(Sqlite))]
pub struct Session {
    pub token: String,
    pub user_id: i32,
    pub expires_at: String,
    pub created_at: Option<String>,
}
