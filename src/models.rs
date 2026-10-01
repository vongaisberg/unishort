use serde_derive::Serialize;

#[derive(Queryable, Serialize)]
pub struct Url {
    pub id: i32,
    pub url: String,
    pub short_url: String,
    pub timestamp: chrono::NaiveDateTime,
    pub public: bool,
    pub clicks: i64,
}
