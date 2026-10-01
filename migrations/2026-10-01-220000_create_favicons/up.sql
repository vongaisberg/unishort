-- Favicons of link targets, fetched once per domain through Google's
-- favicon service so visitors' browsers do not have to ask Google.
CREATE TABLE favicons (
    domain VARCHAR PRIMARY KEY,
    icon BYTEA NOT NULL,
    content_type VARCHAR NOT NULL,
    fetched_at TIMESTAMP NOT NULL
);
