//! Universes Beyond mapping: which printings are out of universe.
//!
//! A printing is out of universe when it carries the triangle security stamp
//! OR its set code appears in [`OUT_OF_UNIVERSE_SETS`]. The stamp alone is not
//! enough: WotC retired it starting with Marvel's Spider-Man (2025-09), so the
//! newest UB tentpoles print with oval or no stamp, and even stamped UB sets
//! hold a few stampless stragglers. The list alone is not enough either: mixed
//! sets (Secret Lair Drop, Ravnica: Clue Edition, promo grab-bags) hold UB
//! cards next to in-universe ones, and there the per-card stamp decides. The
//! pre-stamp crossover Lairs (Walking Dead, Stranger Things, Street Fighter,
//! Fortnite) need no special handling: Scryfall records them triangle-stamped
//! under their canonical Universes Within names, with the UB originals kept
//! only as flavor names.
//!
//! Maintenance: this list is hand-kept and must gain the set codes of each
//! wholly-UB set as it releases (a few times a year). "Recent and stampless"
//! does not imply UB: The Zeta Set (slz) is a stampless in-universe reprint
//! box, so each addition is a judgment call. Census query to find candidates,
//! run against the synced catalog:
//!
//! ```sql
//! SELECT set, set_name, set_type, min(released_at), count(*)
//! FROM scryfall_data GROUP BY 1, 2, 3
//! HAVING count(*) FILTER (WHERE security_stamp = 'triangle') = 0
//!    AND min(released_at) > '<last census date>'
//! ORDER BY 4;
//! ```
//!
//! Consumers: the client's in-memory filter matching, zerver's search
//! predicates, and (via the `oou_sets` table zervice overlays each sync) the
//! `latest_cards` materialized view's printing-preference ORDER BY.

/// Set codes of wholly Universes Beyond sets, satellites included, grouped by
/// franchise. Mixed sets are deliberately absent; their UB cards are caught by
/// the triangle stamp instead.
#[rustfmt::skip]
pub const OUT_OF_UNIVERSE_SETS: &[&str] = &[
    // The Lord of the Rings: Tales of Middle-earth (2023)
    "ltr", "ltc", "pltr", "tltr", "tltc", "pltc",
    // Doctor Who (2023)
    "who", "twho",
    // Fallout (2024)
    "pip", "tpip",
    // Warhammer 40,000 (2022)
    "40k", "t40k",
    // Final Fantasy (2025)
    "fin", "fic", "fca", "pfin", "tfin", "tfic", "wfin", "rfin", "pss5",
    // Assassin's Creed (2024)
    "acr", "tacr",
    // Jurassic World Collection (2023)
    "rex", "trex",
    // Transformers (2022)
    "bot", "tbot",
    // Marvel's Spider-Man (2025, stampless from here on)
    "spm", "spe", "pspm", "mar", "lmar", "aspm", "tspm",
    // Avatar: The Last Airbender (2025)
    "tla", "tle", "ptla", "ttla", "ttle", "atla", "atle", "ftla", "jtla",
    // Marvel Super Heroes (2026)
    "msh", "msc", "tmsh", "tmsc", "amsh", "fmsc",
    // Teenage Mutant Ninja Turtles (2026)
    "tmt", "tmc", "pza", "ttmt", "ttmc", "ftmc", "atmt",
    // The Hobbit (2026)
    "hob", "hoc", "thob",
    // Star Trek (2026)
    "trk", "trc", "ttrk", "sds",
];

/// Returns `true` if a printing with this security stamp and set code is out
/// of universe (Universes Beyond).
pub fn is_out_of_universe(security_stamp: Option<&str>, set: &str) -> bool {
    security_stamp == Some("triangle") || OUT_OF_UNIVERSE_SETS.contains(&set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_stamp_is_out_of_universe_in_any_set() {
        // A UB card inside the mixed Secret Lair Drop set.
        assert!(is_out_of_universe(Some("triangle"), "sld"));
    }

    #[test]
    fn listed_sets_are_out_of_universe_without_a_stamp() {
        // Spider-Man and Avatar print without the triangle stamp.
        assert!(is_out_of_universe(None, "spm"));
        assert!(is_out_of_universe(Some("oval"), "tla"));
    }

    #[test]
    fn in_universe_printings_pass() {
        assert!(!is_out_of_universe(Some("oval"), "sld"));
        assert!(!is_out_of_universe(None, "slz"));
        assert!(!is_out_of_universe(Some("oval"), "slx"));
        assert!(!is_out_of_universe(None, "ecl"));
    }

    #[test]
    fn list_is_lowercase_and_deduplicated() {
        let mut seen = std::collections::HashSet::new();
        for code in OUT_OF_UNIVERSE_SETS {
            assert_eq!(*code, code.to_lowercase(), "set codes are lowercase");
            assert!(seen.insert(*code), "duplicate set code: {code}");
        }
    }
}
