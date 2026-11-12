//! Usernames no one can pick: Scotty's own names, the brand, and the words the sites and the API
//! use as paths or that read as staff or the system.
//!
//! zerver checks a new username against this list on register and on a username change, and
//! answers a reserved one exactly as it answers a taken one. Accounts that already hold a name
//! keep it; only new names are checked. Scotty claims a reserved name from the server, with
//! `zerver-admin set-username`, which skips the check.
//!
//! A name matches a reserved word after [`fold`]: lowercase, with `.`, `-`, `_` and whitespace
//! dropped and common lookalikes folded (`0` reads as `o`, `1` and `l` as `i`, `3` as `e`, `4` and
//! `@` as `a`, `5` and `$` as `s`, `7` as `t`). A reserved word of four letters or more also
//! matches with a number after it or `official`, `real` or `the` around it (`zwipe1`, `realzwipe`,
//! `thezwipe`, `zwipeofficial`); shorter words match only as themselves, so `theme` stays open.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Scotty's names and nicknames.
const SCOTTY: &[&str] = &[
    "scadoshi",
    "scotty",
    "scott",
    "scotland",
    "scooter",
    "fermo",
    "scottyfermo",
    "scottfermo",
];

/// The brand, its apps and crates, and the names it is known by or could be mistaken for.
const BRAND: &[&str] = &[
    "zwipe",
    "zwipes",
    "zwiper",
    "zwipey",
    "zipe",
    "swipe",
    "swiper",
    "swipes",
    "zite",
    "zerver",
    "zervice",
    "zwiper-web",
    "zwipe-core",
    "zwipe-ui",
    "zynergy",
    "zynergizer",
    "synergy",
    "cairn",
    "orangui",
    "heron",
    "steller",
];

/// The data and the game Zwipe is built on, whose staff a name could pose as.
const PARTNERS: &[&str] = &["scryfall", "wizards", "wotc", "archidekt", "moxfield"];

/// The top-level paths of app.zwipe.net and zwipe.net, now and planned. A shared deck lives at
/// `/<owner>/deck/...`, so an owner's name sits where these do.
const ROUTES: &[&str] = &[
    "about",
    "account",
    "activity",
    "android",
    "cards",
    "changelog",
    "community",
    "config",
    "confirm-email",
    "contribute",
    "deck",
    "decks",
    "discord",
    "download",
    "downloads",
    "explore",
    "forgot-password",
    "guides",
    "home",
    "ios",
    "login",
    "logout",
    "maybeboard",
    "playstyle",
    "privacy",
    "profile",
    "profiles",
    "register",
    "reset",
    "search",
    "settings",
    "share",
    "shared",
    "signin",
    "signout",
    "signup",
    "sitemap",
    "robots",
    "favicon",
    "verify",
    "well-known",
];

/// The API's top-level paths.
const API: &[&str] = &[
    "api",
    "auth",
    "card",
    "client",
    "commander",
    "marketing",
    "metrics",
];

/// Names that read as staff, the system or nobody in particular.
const SYSTEM: &[&str] = &[
    "admin",
    "administrator",
    "root",
    "support",
    "help",
    "staff",
    "mod",
    "moderator",
    "official",
    "team",
    "system",
    "null",
    "undefined",
    "none",
    "nil",
    "true",
    "false",
    "me",
    "you",
    "user",
    "users",
    "everyone",
    "anonymous",
    "guest",
    "nobody",
    "unknown",
    "deleted",
];

/// Environments and infrastructure.
const INFRASTRUCTURE: &[&str] = &[
    "app",
    "www",
    "dev",
    "development",
    "staging",
    "stage",
    "prod",
    "production",
    "test",
    "testing",
    "beta",
    "alpha",
    "demo",
    "sandbox",
    "localhost",
    "status",
    "health",
    "uptime",
    "docs",
    "blog",
    "cdn",
    "static",
    "assets",
    "media",
    "mail",
    "email",
    "smtp",
    "ftp",
    "ssh",
    "git",
    "github",
    "webhook",
    "webhooks",
    "oauth",
    "sso",
    "bot",
    "bots",
    "service",
    "server",
    "internal",
    "debug",
    "sysadmin",
    "security",
    "database",
    "backup",
    "dashboard",
    "console",
    "cloudflare",
    "resend",
];

/// Business and legal addresses.
const BUSINESS: &[&str] = &[
    "billing",
    "legal",
    "terms",
    "tos",
    "dmca",
    "abuse",
    "noreply",
    "no-reply",
    "postmaster",
    "hostmaster",
    "webmaster",
    "info",
    "contact",
    "sales",
    "press",
    "careers",
    "jobs",
    "store",
    "shop",
    "buy",
    "cart",
];

/// Every group, in the order above.
pub const RESERVED_GROUPS: [&[&str]; 8] = [
    SCOTTY,
    BRAND,
    PARTNERS,
    ROUTES,
    API,
    SYSTEM,
    INFRASTRUCTURE,
    BUSINESS,
];

/// Words that also wrap a reserved word: before it, or `official` after it.
const PREFIXES: [&str; 3] = ["official", "real", "the"];

/// Folded reserved words shorter than this match only as themselves.
const MIN_WRAPPED_LEN: usize = 4;

/// The reserved words, folded.
static RESERVED: LazyLock<HashSet<String>> = LazyLock::new(|| {
    RESERVED_GROUPS
        .iter()
        .flat_map(|group| group.iter())
        .map(|word| fold(word))
        .collect()
});

/// `name` lowercased, with `.`, `-`, `_` and whitespace dropped and lookalikes folded, so two names
/// a reader would take for the same fold to the same string.
#[must_use]
pub fn fold(name: &str) -> String {
    squash(name).chars().map(fold_char).collect()
}

/// `name` lowercased with `.`, `-`, `_` and whitespace dropped, lookalikes kept.
fn squash(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '.' | '-' | '_') && !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The letter a lookalike stands for.
fn fold_char(c: char) -> char {
    match c {
        '0' => 'o',
        '1' | 'l' | '!' | '|' => 'i',
        '3' => 'e',
        '4' | '@' => 'a',
        '5' | '$' => 's',
        '7' => 't',
        other => other,
    }
}

/// Whether `name` is reserved: a reserved word after [`fold`], or one of four letters or more with
/// a number after it or `official`, `real` or `the` around it.
#[must_use]
pub fn is_reserved(name: &str) -> bool {
    let squashed = squash(name);
    let unnumbered = squashed.trim_end_matches(|c: char| c.is_ascii_digit());
    let folded = fold(&squashed);
    if RESERVED.contains(&folded) {
        return true;
    }
    let mut cores = vec![fold(unnumbered)];
    let mut at = 0;
    while let Some(core) = cores.get(at).cloned() {
        at = at.saturating_add(1);
        for prefix in PREFIXES.map(fold) {
            if let Some(rest) = core.strip_prefix(&prefix) {
                cores.push(rest.to_owned());
            }
        }
        if let Some(rest) = core.strip_suffix(&fold("official")) {
            cores.push(rest.to_owned());
        }
    }
    cores
        .iter()
        .any(|core| core.chars().count() >= MIN_WRAPPED_LEN && RESERVED.contains(core))
}

/// Whether `new` folds to the same name as `current`, so an account may change only how its own
/// name is written even when that name is reserved.
#[must_use]
pub fn same_name(current: &str, new: &str) -> bool {
    fold(current) == fold(new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_names_match_in_any_case_and_with_separators() {
        for name in [
            "zwipe",
            "Zwipe",
            "ZWIPE",
            "z.wipe",
            "z-w_i.p-e",
            "Scotty_Fermo",
            "scotty.fermo",
            "scotty-fermo",
            "No-Reply",
            "noreply",
            "zwiper_web",
            "Admin",
        ] {
            assert!(is_reserved(name), "{name}");
        }
    }

    #[test]
    fn lookalikes_fold_to_their_letters() {
        for name in [
            "zw1pe", "zwip3", "sc0tty", "5cotty", "$cotty", "sco77y", "@dmin", "4dmin", "adm1n",
            "he1p", "heip", "c4irn", "0rangui",
        ] {
            assert!(is_reserved(name), "{name}");
        }
    }

    #[test]
    fn a_number_after_or_official_real_or_the_around_a_long_word_match() {
        for name in [
            "zwipe1",
            "zwipe2026",
            "scotty_99",
            "realzwipe",
            "thezwipe",
            "officialzwipe",
            "therealzwipe",
            "zwipeofficial",
            "the_real_scotty",
            "real_sc0tty7",
            "admin1",
        ] {
            assert!(is_reserved(name), "{name}");
        }
    }

    #[test]
    fn short_words_match_only_as_themselves() {
        assert!(is_reserved("app"));
        assert!(is_reserved("mod"));
        for name in ["theme", "realme", "theapp", "mod1", "app2", "api3", "you2"] {
            assert!(!is_reserved(name), "{name}");
        }
    }

    #[test]
    fn ordinary_names_stay_open() {
        for name in [
            "goblinking",
            "krenko_fan",
            "scottish",
            "swipeleft",
            "zwipefan",
            "heronsong",
            "mtgdad",
            "alice",
            "bob_the_builder",
            "theodore",
        ] {
            assert!(!is_reserved(name), "{name}");
        }
    }

    #[test]
    fn every_reserved_word_is_reserved_as_written() {
        for word in RESERVED_GROUPS.iter().flat_map(|group| group.iter()) {
            assert!(is_reserved(word), "{word}");
        }
    }

    #[test]
    fn an_account_may_rewrite_its_own_name() {
        assert!(same_name("Scotty", "scotty"));
        assert!(same_name("scotty", "Sc0tty"));
        assert!(!same_name("scotty", "scott"));
    }
}
