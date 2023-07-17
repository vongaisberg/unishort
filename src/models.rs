use crate::schema::urls;
use diesel::Insertable;
use serde_derive::Serialize;
use std::time::SystemTime;

#[derive(Queryable, Insertable, Serialize)]
pub struct Url {
    pub id: i32,
    pub url: String,
    pub short_url: String,
    pub timestamp: chrono::NaiveDateTime,
    pub public: bool,
    pub clicks: i64,
}
