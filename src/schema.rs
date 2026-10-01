// @generated automatically by Diesel CLI.

diesel::table! {
    favicons (domain) {
        domain -> Varchar,
        icon -> Bytea,
        content_type -> Varchar,
        fetched_at -> Timestamp,
    }
}

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

diesel::allow_tables_to_appear_in_same_query!(favicons, urls);
