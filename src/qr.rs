//! QR codes for short links, rendered here instead of by a third party.

use qrcode::render::svg;
use qrcode::QrCode;
use rocket::http::{ContentType, Header};
use rocket::State;

use crate::ServerUrl;

/// Longer than any short code we hand out (two code points), short enough
/// that nobody can make us render large codes for arbitrary text.
const MAX_CODE_CHARS: usize = 8;

#[derive(Responder)]
pub struct CachedSvg {
    inner: (ContentType, String),
    cache_control: Header<'static>,
}

/// The QR code for `data` as an SVG document.
pub fn render(data: &str) -> Option<String> {
    let code = QrCode::new(data.as_bytes()).ok()?;
    Some(
        code.render::<svg::Color>()
            .min_dimensions(150, 150)
            .quiet_zone(true)
            .build(),
    )
}

/// The QR code for a short link. It encodes the same text the third-party
/// QR service used to get: the display URL and the code, unencoded.
#[get("/qr/<code>")]
pub fn qr(code: &str, server_url: &State<ServerUrl>) -> Option<CachedSvg> {
    if code.chars().count() > MAX_CODE_CHARS {
        return None;
    }
    let svg = render(&format!("{}/{}", server_url.0, code))?;
    Some(CachedSvg {
        inner: (ContentType::SVG, svg),
        // A short link never changes what it points to, so neither does its
        // QR code.
        cache_control: Header::new("Cache-Control", "public, max-age=31536000, immutable"),
    })
}
