//! The last theme, kept in browser storage so the login screen opens in it.
//!
//! Unlike the session, it stays after logout.

use components::ThemeConfig;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the theme lives under.
pub const THEME_KEY: &str = "zwiper-web.theme";

/// Saves and loads the last theme in `S`.
#[derive(Debug, Clone, Default)]
pub struct ThemeStore<S> {
    storage: S,
}

impl ThemeStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> ThemeStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The stored theme. `None` when there is none, when it does not parse,
    /// or when it names no theme this build has; a retired theme loads as
    /// its replacement.
    pub fn load(&self) -> Option<ThemeConfig> {
        let json = self.storage.read(THEME_KEY)?;
        serde_json::from_str::<ThemeConfig>(&json)
            .ok()
            .and_then(ThemeConfig::resolve)
    }

    /// Stores `theme`, logging a failure. The theme is a convenience, so a
    /// lost write costs only the next visit's first look.
    pub fn save(&self, theme: &ThemeConfig) {
        let result = serde_json::to_string(theme)
            .map_err(|e| e.to_string())
            .and_then(|json| {
                self.storage
                    .write(THEME_KEY, &json)
                    .map_err(|e| e.to_string())
            });
        if let Err(e) = result {
            tracing::debug!("theme not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_theme_loads_back() {
        let store = ThemeStore::new(MemoryStorage::default());
        let theme = ThemeConfig {
            name: "nord".to_string(),
            is_dark: false,
        };
        store.save(&theme);
        assert_eq!(store.load(), Some(theme));
    }

    #[test]
    fn an_unknown_theme_loads_none() {
        let store = ThemeStore::new(MemoryStorage::default());
        store.save(&ThemeConfig {
            name: "a-theme-from-a-newer-build".to_string(),
            is_dark: true,
        });
        assert_eq!(store.load(), None);
    }

    #[test]
    fn garbage_loads_none() {
        let storage = MemoryStorage::default();
        storage.write(THEME_KEY, "nope").unwrap();
        assert_eq!(ThemeStore::new(storage).load(), None);
    }

    /// The page shell, whose first-paint script reads this store's key.
    const INDEX_HTML: &str = include_str!("../../../index.html");

    /// The response headers, whose CSP names that script's hash.
    const HEADERS: &str = include_str!("../../../deploy/_headers");

    /// `text` between the first `start` and the next `end` after it.
    fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
        let (_, after) = text.split_once(start).expect("start marker");
        let (inside, _) = after.split_once(end).expect("end marker");
        inside
    }

    /// Standard base64 with padding, as a CSP hash is written.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        reason = "bit arithmetic on bytes and 6-bit indexes that cannot overflow"
    )]
    fn base64(bytes: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let byte = |i: usize| u32::from(chunk.get(i).copied().unwrap_or(0));
            let n = (byte(0) << 16) | (byte(1) << 8) | byte(2);
            for i in 0..4 {
                if i <= chunk.len() {
                    let index = ((n >> (18 - 6 * i)) & 63) as usize;
                    out.push(char::from(*ALPHABET.get(index).expect("a 6-bit index")));
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    #[test]
    fn first_paint_matches_the_themes() {
        use sha2::{Digest, Sha256};

        assert!(
            INDEX_HTML.contains(&format!("localStorage.getItem(\"{THEME_KEY}\")")),
            "index.html reads another key than {THEME_KEY}"
        );

        // Every theme's page color, in step with orangui's themes.css.
        let colors: std::collections::HashMap<String, String> =
            serde_json::from_str(between(INDEX_HTML, "var colors = ", ";\n"))
                .expect("the color map");
        assert_eq!(colors.len(), components::ALLOWED_THEMES.len() * 2);
        for theme in components::ALLOWED_THEMES {
            for mode in ["dark", "light"] {
                let block = between(
                    components::THEMES_CSS,
                    &format!(".theme-{theme}-{mode} {{"),
                    "}",
                );
                let color = between(block, "--bg-primary:", ";").trim().to_lowercase();
                assert_eq!(
                    colors.get(&format!("{theme}-{mode}")),
                    Some(&color),
                    "{theme}-{mode}'s first paint differs from themes.css"
                );
            }
        }

        // html in the page color, the footer's, and body in the sink, mixed
        // as themes.css mixes each mode's.
        let style = between(INDEX_HTML, "<style>", "</style>");
        let rule = |selector: &str| {
            between(style, &format!("{selector} {{"), "}")
                .trim()
                .to_owned()
        };
        assert_eq!(
            rule(":where(html)"),
            "background-color: var(--first-paint-bg, #282828);"
        );
        let dark_sink = rule(":where(body)");
        let light_sink = rule(":where(html.first-paint-light body)");
        assert!(
            dark_sink.contains("color-mix(in srgb, var(--first-paint-bg, #282828), #000 15%)")
                && components::THEMES_CSS
                    .contains("--bg-sink: color-mix(in srgb, var(--bg-primary), #000 15%);"),
            "the dark first paint's sink differs from themes.css: {dark_sink}"
        );
        assert!(
            light_sink.contains("color-mix(in srgb, var(--first-paint-bg), #fff 35%)")
                && components::THEMES_CSS
                    .contains("--bg-sink: color-mix(in srgb, var(--bg-primary), #fff 35%);"),
            "the light first paint's sink differs from themes.css: {light_sink}"
        );

        // The retired slugs, as the store reads them.
        for (from, to) in components::RETIRED_THEMES {
            assert!(
                INDEX_HTML.contains(&format!("\"{from}\": \"{to}\"")),
                "{from} -> {to}"
            );
        }

        // The CSP lets this exact script run.
        let script = between(INDEX_HTML, "<script>", "</script>");
        let hash = base64(&Sha256::digest(script.as_bytes()));
        assert!(
            HEADERS.contains(&format!("'sha256-{hash}'")),
            "deploy/_headers' script-src lacks 'sha256-{hash}'"
        );
    }
}
