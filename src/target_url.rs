//! Turning what people type into the URL a short link redirects to, and back
//! into something readable for the list of recent links.

use regex::Regex;
use std::borrow::Cow;
use url::{Host, Position, Url};

lazy_static! {
    /// An explicit scheme, in any case. Without one, the input is taken to be
    /// a scheme-less URL like `example.com/page`.
    static ref SCHEME_REGEX: Regex = Regex::new(r"^[A-Za-z][A-Za-z0-9+.\-]*://").unwrap();
}

/// Parses user input into an http(s) URL, adding `https://` when no scheme
/// was given. Returns `None` for anything a short link should not point to.
///
/// The result is normalized by the `url` crate: scheme and host are
/// lowercased, international domains become punycode, and unsafe characters
/// are percent-encoded. Path, query and fragment keep their case.
pub fn normalize(input: &str) -> Option<Url> {
    let input = input.trim();
    let with_scheme = if SCHEME_REGEX.is_match(input) {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(format!("https://{}", input))
    };
    let parsed = Url::parse(&with_scheme).ok()?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    // `https://google.com@evil.com` goes to evil.com. Credentials in a short
    // link's target are almost always that trick, and turning `mailto:a@b.c`
    // into `https://mailto:a@b.c` would be one by accident.
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return None;
    }
    match parsed.host()? {
        Host::Ipv4(_) | Host::Ipv6(_) => {}
        // A public domain: at least two labels and a top-level domain that is
        // not a number. This rejects `localhost` and other single-label names.
        Host::Domain(domain) => {
            let domain = domain.trim_end_matches('.');
            let (_, tld) = domain.rsplit_once('.')?;
            if tld.len() < 2 || tld.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
        }
    }
    Some(parsed)
}

/// `target` with its domain shown in Unicode rather than punycode, for
/// display only. Anything that does not parse is returned unchanged.
pub fn display(target: &str) -> String {
    let parsed = match Url::parse(target) {
        Ok(parsed) => parsed,
        Err(_) => return target.to_owned(),
    };
    match parsed.host() {
        Some(Host::Domain(domain)) => match idna::domain_to_unicode(domain) {
            (unicode, Ok(())) => format!(
                "{}{}{}",
                &parsed[..Position::BeforeHost],
                unicode,
                &parsed[Position::AfterHost..]
            ),
            _ => target.to_owned(),
        },
        _ => target.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(input: &str) -> Option<String> {
        normalize(input).map(String::from)
    }

    #[test]
    fn adds_https_to_scheme_less_input() {
        assert_eq!(norm("example.com").as_deref(), Some("https://example.com/"));
        assert_eq!(norm("  example.com/a b  ").as_deref(), Some("https://example.com/a%20b"));
        assert_eq!(norm("example.com:8443/x").as_deref(), Some("https://example.com:8443/x"));
    }

    #[test]
    fn keeps_explicit_http_and_https_in_any_case() {
        assert_eq!(norm("http://example.com").as_deref(), Some("http://example.com/"));
        assert_eq!(norm("HTTPS://Example.COM/x").as_deref(), Some("https://example.com/x"));
    }

    #[test]
    fn preserves_case_of_path_query_and_fragment() {
        assert_eq!(
            norm("youtube.com/watch?v=dQw4w9WgXcQ#T").as_deref(),
            Some("https://youtube.com/watch?v=dQw4w9WgXcQ#T")
        );
    }

    #[test]
    fn stores_international_domains_as_punycode() {
        assert_eq!(norm("müller.de").as_deref(), Some("https://xn--mller-kva.de/"));
        assert_eq!(norm("🤏.to/x").as_deref(), Some("https://xn--rp9h.to/x"));
    }

    #[test]
    fn accepts_ip_addresses() {
        assert_eq!(norm("192.168.1.1/x").as_deref(), Some("https://192.168.1.1/x"));
        assert_eq!(norm("http://[::1]:8080/").as_deref(), Some("http://[::1]:8080/"));
    }

    #[test]
    fn rejects_other_schemes() {
        assert_eq!(norm("ftp://example.com"), None);
        assert_eq!(norm("javascript:alert(1)"), None);
        assert_eq!(norm("JavaScript://example.com/%0aalert(1)"), None);
        assert_eq!(norm("data:text/html,<script>alert(1)</script>"), None);
    }

    #[test]
    fn rejects_credentials() {
        assert_eq!(norm("mailto:a@example.com"), None);
        assert_eq!(norm("https://google.com@evil.com"), None);
        assert_eq!(norm("user:pass@example.com"), None);
    }

    #[test]
    fn rejects_non_public_hostnames_and_junk() {
        assert_eq!(norm("localhost:3000"), None);
        assert_eq!(norm("intranet"), None);
        assert_eq!(norm("example.c"), None);
        assert_eq!(norm(""), None);
        assert_eq!(norm("not a url"), None);
    }

    #[test]
    fn displays_domains_in_unicode() {
        assert_eq!(display("https://xn--mller-kva.de/Path?q=1"), "https://müller.de/Path?q=1");
        assert_eq!(display("https://xn--rp9h.to/"), "https://🤏.to/");
        assert_eq!(display("https://example.com/"), "https://example.com/");
        assert_eq!(display("http://192.168.1.1:8080/"), "http://192.168.1.1:8080/");
        assert_eq!(display("not a url"), "not a url");
    }
}
