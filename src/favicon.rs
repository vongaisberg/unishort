//! Favicons for the recent links table, served from our own origin.
//!
//! Visitors' browsers used to load these from Google directly, which told
//! Google who was looking at which links. Now this server asks Google's
//! favicon service once per domain and keeps the result in Postgres, so
//! Google learns each linked domain once and nothing about visitors.
//!
//! Fetching through Google rather than from the linked sites themselves is
//! deliberate: link targets are user input, and fetching them directly would
//! let anyone make this server request addresses inside the cluster.

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use rocket::http::{ContentType, Header, Status};
use rocket::State;
use std::time::Duration as StdDuration;

use crate::db::{self, PgPool};
use crate::schema::favicons;
use crate::schema::urls;

/// Larger answers are not favicons; Google's are around 1 KB.
const MAX_ICON_BYTES: usize = 64 * 1024;
/// How long a stored icon is served before it is fetched again.
const MAX_AGE_DAYS: i64 = 30;

pub struct Client(reqwest::Client);

impl Client {
    pub fn new() -> Self {
        // Google answers with a redirect to gstatic.com. Follow only that,
        // so a redirect can never point this server anywhere else.
        let redirects = reqwest::redirect::Policy::custom(|attempt| {
            let host = attempt.url().host_str().unwrap_or("");
            let allowed = ["google.com", "gstatic.com"]
                .iter()
                .any(|d| host == *d || host.ends_with(&format!(".{}", d)));
            if allowed && attempt.previous().len() < 5 {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });
        Client(
            reqwest::Client::builder()
                .redirect(redirects)
                .timeout(StdDuration::from_secs(5))
                .build()
                .expect("Could not build the favicon HTTP client"),
        )
    }
}

#[derive(Queryable)]
struct Favicon {
    icon: Vec<u8>,
    content_type: String,
    fetched_at: NaiveDateTime,
}

fn is_fresh(favicon: &Favicon) -> bool {
    favicon.fetched_at > Utc::now().naive_utc() - Duration::days(MAX_AGE_DAYS)
}

#[derive(Responder)]
pub struct CachedIcon {
    inner: (ContentType, Vec<u8>),
    cache_control: Header<'static>,
}

impl CachedIcon {
    fn new(content_type: &str, icon: Vec<u8>) -> Self {
        CachedIcon {
            inner: (
                ContentType::parse_flexible(content_type).unwrap_or(ContentType::PNG),
                icon,
            ),
            cache_control: Header::new("Cache-Control", "public, max-age=86400"),
        }
    }
}

/// `domain` is a link target's host as the `url` crate serializes it:
/// lowercase ASCII, punycode for international domains, brackets for IPv6.
fn is_valid_host(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || ".-_:[]".contains(c))
}

/// Whether some stored link points at `domain`. Only those get fetched, so
/// this cannot be used as an open proxy to Google, or to fill the table.
fn is_linked(conn: &PgConnection, domain: &str) -> QueryResult<bool> {
    // Rows from before URLs were normalized can hold the Unicode form.
    let (unicode, _) = idna::domain_to_unicode(domain);
    let mut patterns = Vec::new();
    for host in [domain, unicode.as_str()] {
        let host = host.replace('\\', "\\\\").replace('_', "\\_").replace('%', "\\%");
        for scheme in ["http", "https"] {
            patterns.push(format!("{}://{}/%", scheme, host));
            patterns.push(format!("{}://{}:%", scheme, host));
            patterns.push(format!("{}://{}", scheme, host));
        }
    }
    let mut query = urls::table.select(urls::id).into_boxed();
    for pattern in patterns {
        query = query.or_filter(urls::url.like(pattern));
    }
    query.first::<i32>(conn).optional().map(|id| id.is_some())
}

/// Asks Google for `domain`'s favicon. Google answers 404 with a generic
/// globe for domains it has no icon for, which is the right thing to show.
async fn fetch(client: &Client, domain: &str) -> Result<(String, Vec<u8>), String> {
    let mut response = client
        .0
        .get("https://www.google.com/s2/favicons")
        .query(&[("domain", domain), ("sz", "32")])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    // Raster images only: an SVG is a document and could carry script.
    if !content_type.starts_with("image/") || content_type.starts_with("image/svg") {
        return Err(format!(
            "unexpected answer {} {:?}",
            response.status(),
            content_type
        ));
    }
    let mut icon = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        icon.extend_from_slice(&chunk);
        if icon.len() > MAX_ICON_BYTES {
            return Err("icon too large".to_owned());
        }
    }
    Ok((content_type, icon))
}

#[get("/favicon/<domain>")]
pub async fn favicon(
    domain: &str,
    pool: &State<PgPool>,
    client: &State<Client>,
) -> Result<CachedIcon, Status> {
    if !is_valid_host(domain) {
        return Err(Status::NotFound);
    }
    let owned = domain.to_owned();
    let (stored, linked) = db::run(pool, move |conn| {
        let stored = favicons::table
            .select((favicons::icon, favicons::content_type, favicons::fetched_at))
            .find(&owned)
            .first::<Favicon>(conn)
            .optional()?;
        // Only needed when there is nothing fresh to serve.
        let linked = stored.as_ref().map_or(false, is_fresh) || is_linked(conn, &owned)?;
        Ok::<_, diesel::result::Error>((stored, linked))
    })
    .await?
    .map_err(|e| {
        log::error!("Could not look up favicon: {}", e);
        Status::InternalServerError
    })?;

    if let Some(f) = stored.as_ref().filter(|f| is_fresh(f)) {
        return Ok(CachedIcon::new(&f.content_type, f.icon.clone()));
    }
    if !linked {
        return Err(Status::NotFound);
    }

    match fetch(client, domain).await {
        Ok((content_type, icon)) => {
            let row = (
                favicons::domain.eq(domain.to_owned()),
                favicons::icon.eq(icon.clone()),
                favicons::content_type.eq(content_type.clone()),
                favicons::fetched_at.eq(Utc::now().naive_utc()),
            );
            let saved = db::run(pool, move |conn| {
                diesel::insert_into(favicons::table)
                    .values(&row)
                    .on_conflict(favicons::domain)
                    .do_update()
                    .set((
                        favicons::icon.eq(excluded(favicons::icon)),
                        favicons::content_type.eq(excluded(favicons::content_type)),
                        favicons::fetched_at.eq(excluded(favicons::fetched_at)),
                    ))
                    .execute(conn)
            })
            .await;
            if let Ok(Err(e)) = saved {
                log::error!("Could not store favicon: {}", e);
            }
            Ok(CachedIcon::new(&content_type, icon))
        }
        // A stale icon beats none while Google is unreachable.
        Err(e) => {
            log::warn!("Could not fetch favicon for {}: {}", domain, e);
            stored
                .map(|f| CachedIcon::new(&f.content_type, f.icon))
                .ok_or(Status::NotFound)
        }
    }
}
