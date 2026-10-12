-- Fails if the same URL has been shortened more than once since.
ALTER TABLE urls ADD CONSTRAINT urls_url_key UNIQUE (url);
