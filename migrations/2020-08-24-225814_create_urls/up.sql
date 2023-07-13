CREATE TABLE urls (
    id SERIAL PRIMARY KEY,
    url VARCHAR UNIQUE NOT NULL,
    short_url VARCHAR UNIQUE NOT NULL,
    timestamp TIMESTAMP NOT NULL
  );
CREATE INDEX short_url_index ON urls (short_url);
CREATE INDEX url_index ON urls (url);