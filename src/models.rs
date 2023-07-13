use std::time::SystemTime;
use diesel::Insertable;
use crate::schema::urls;

#[derive(Queryable, Insertable)]
pub struct Url {
    pub id: i32,
    pub url: String,
    pub short_url: String,
    pub timestamp: SystemTime,
}