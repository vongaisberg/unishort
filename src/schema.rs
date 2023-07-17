// @generated automatically by Diesel CLI.

diesel::table! {
    urls (id) {
        id -> Int4,
        url -> Varchar,
        short_url -> Varchar,
        timestamp -> Timestamp,
        public -> Bool,
        clicks -> Int8,
    }
}
