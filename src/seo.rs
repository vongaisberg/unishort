//! What search engines see: one canonical site at pinch.to.
//!
//! The same app answers on three hosts. pinch.to is the site; 🤏.to is the
//! domain short links are made on, so its pages move to pinch.to; and
//! app-unishort.vongaisberg.de (or any other host) is internal, so it is kept
//! out of search results instead of competing with pinch.to.

use rocket::http::{ContentType, Header, Method, Status};
use rocket::{Request, Response};

pub const CANONICAL_HOST: &str = "pinch.to";
/// 🤏.to as it arrives in the Host header.
const SHORT_LINK_HOST: &str = "xn--rp9h.to";

const ROBOTS_TXT: &str = "User-agent: *
Disallow: /favicon/
Disallow: /qr/

Sitemap: https://pinch.to/sitemap.xml
";

const SITEMAP_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://pinch.to/</loc></url>
  <url><loc>https://pinch.to/blog/about</loc></url>
  <url><loc>https://pinch.to/blog/top-10-shortest-url-shorteners-2023</loc></url>
</urlset>
"#;

#[get("/robots.txt")]
pub fn robots() -> (ContentType, &'static str) {
    (ContentType::Text, ROBOTS_TXT)
}

#[get("/sitemap.xml")]
pub fn sitemap() -> (ContentType, &'static str) {
    (ContentType::XML, SITEMAP_XML)
}

/// Applied to every HTML response. Short-link redirects are not HTML, so
/// 🤏.to/ꙮ𐐀 keeps working on every host.
pub fn on_page(request: &Request<'_>, response: &mut Response<'_>) {
    let host = request
        .host()
        .map(|h| h.domain().as_str().to_ascii_lowercase());
    match host.as_deref() {
        Some(CANONICAL_HOST) => {}
        Some(SHORT_LINK_HOST) => {
            if request.method() == Method::Get && response.status() == Status::Ok {
                let location = format!("https://{}{}", CANONICAL_HOST, request.uri().path());
                *response = Response::build()
                    .status(Status::MovedPermanently)
                    .header(Header::new("Location", location))
                    .finalize();
            }
        }
        _ => {
            response.set_header(Header::new("X-Robots-Tag", "noindex"));
        }
    }
}
