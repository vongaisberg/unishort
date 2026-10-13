//! Versioned URLs for the stylesheet and script.
//!
//! Cloudflare keeps static files for hours, so after a deploy it would pair
//! the new pages with the old stylesheet. `{{asset "styles.css"}}` gives
//! `/static/styles.css?v=<hash of its content>`, a new URL whenever the file
//! changes.

use rocket_dyn_templates::handlebars::handlebars_helper;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};

lazy_static! {
    static ref VERSIONS: HashMap<String, String> = fs::read_dir("static")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let content = fs::read(entry.path()).ok()?;
            Some((name, version_of(&content)))
        })
        .collect();
}

fn version_of(content: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:08x}", hasher.finish() as u32)
}

/// The URL of `name` in the static directory, with its version.
pub fn url(name: &str) -> String {
    match VERSIONS.get(name) {
        Some(version) => format!("/static/{}?v={}", name, version),
        None => format!("/static/{}", name),
    }
}

handlebars_helper!(asset: |name: str| url(name));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_change_with_content() {
        assert_eq!(version_of(b"a"), version_of(b"a"));
        assert_ne!(version_of(b"a"), version_of(b"b"));
        assert!(url("styles.css").starts_with("/static/styles.css?v="));
        assert_eq!(url("missing.css"), "/static/missing.css");
    }
}
