# unishort

A URL shortener that uses (almost) the full Unicode range to make short links
of two code points, like `🤏.to/ꙮ𐐀`. Hosted at [🤏.to](https://🤏.to).

Rust (Rocket, Diesel, Postgres). Pages load nothing from third parties: QR
codes are rendered here and favicons are proxied and stored in Postgres.

## Running locally

```sh
export DATABASE_URL=postgres://user:pass@localhost/unishort URL=http://localhost:8000
cargo run      # applies pending migrations at startup
cargo test
```

## Deployment

Runs on the `sun` k3s cluster. A push builds the image
(`.github/workflows/build-deploy.yml`), pushes it to the Zot registry and
commits the tag to [sun-gitops](https://github.com/vongaisberg/sun-gitops),
where Argo CD rolls it out.

- `dev` → `https://app-unishort-dev.vongaisberg.de` (behind Keycloak, `sun`
  realm, `apps-users` group). Check changes here first.
- `master` → `https://app-unishort.vongaisberg.de`. Merge `dev` into it.
