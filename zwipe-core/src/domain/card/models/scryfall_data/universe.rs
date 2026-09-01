//! Universes Beyond mapping: which printings are out of universe.
//!
//! A printing is out of universe when it carries the triangle security stamp
//! OR its set code belongs to a franchise in [`FRANCHISES`]. The stamp alone
//! is not enough: WotC retired it starting with Marvel's Spider-Man (2025-09),
//! so the newest UB tentpoles print with oval or no stamp, and even stamped UB
//! sets hold stampless satellites (art series, minigames, front cards). The
//! list alone is not enough either: mixed sets (Secret Lair Drop, Ravnica:
//! Clue Edition, promo grab-bags) hold UB cards next to in-universe ones, and
//! there the per-card stamp decides. The pre-stamp crossover Lairs (Walking
//! Dead, Stranger Things, Street Fighter, Fortnite) need no special handling:
//! Scryfall records them triangle-stamped under their canonical Universes
//! Within names, with the UB originals kept only as flavor names.
//!
//! Franchises are the user-facing grouping for the exclude-UB preference's
//! exceptions ("exclude UB but let Middle-earth through"): slugs are stored in
//! `user_preferences.universes_beyond_exceptions`, names label the checkboxes,
//! and the (code, name) pairs drive both the SQL predicates (set names match
//! `latest_cards.printing_set_names`) and the nightly `oou_sets` overlay.
//!
//! Maintenance: hand-kept; add each wholly-UB set (satellites included) as it
//! releases, a few times a year. "Recent and stampless" does not imply UB (The
//! Zeta Set is a stampless in-universe reprint box), so each addition is a
//! judgment call. Census query for candidates, against the synced catalog:
//!
//! ```sql
//! SELECT set, set_name, set_type, min(released_at), count(*)
//! FROM scryfall_data GROUP BY 1, 2, 3
//! HAVING min(released_at) > '<last census date>'
//! ORDER BY 4;
//! ```

/// One Universes Beyond franchise: the unit users except from the exclude-UB
/// preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UbFranchise {
    /// Stable identifier stored in user preferences. Never rename.
    pub slug: &'static str,
    /// Display name for the exceptions checkboxes.
    pub name: &'static str,
    /// Every set in the franchise as (set code, set name), satellites
    /// included. Codes drive the oou_sets overlay and pick predicates; names
    /// match `latest_cards.printing_set_names`.
    pub sets: &'static [(&'static str, &'static str)],
}

/// Wholly Universes Beyond sets, grouped by franchise. Mixed sets are
/// deliberately absent; their UB cards are caught by the triangle stamp.
pub const FRANCHISES: &[UbFranchise] = &[
    UbFranchise {
        slug: "middle-earth",
        name: "Middle-earth",
        sets: &[
            ("ltr", "The Lord of the Rings: Tales of Middle-earth"),
            ("ltc", "Tales of Middle-earth Commander"),
            ("pltr", "Tales of Middle-earth Promos"),
            ("tltr", "Tales of Middle-earth Tokens"),
            ("tltc", "Tales of Middle-earth Commander Tokens"),
            ("pltc", "Tales of Middle-earth Deluxe Commander Kit"),
            ("altr", "Tales of Middle-earth Art Series"),
            ("fltr", "Tales of Middle-earth Front Cards"),
            ("altc", "Tales of Middle-earth Scene Box"),
            (
                "mltr",
                "The Lord of the Rings: Tales of Middle-earth Minigames",
            ),
            ("hob", "The Hobbit"),
            ("hoc", "The Hobbit Eternal"),
            ("thob", "The Hobbit Tokens"),
        ],
    },
    UbFranchise {
        slug: "doctor-who",
        name: "Doctor Who",
        sets: &[("who", "Doctor Who"), ("twho", "Doctor Who Tokens")],
    },
    UbFranchise {
        slug: "fallout",
        name: "Fallout",
        sets: &[("pip", "Fallout"), ("tpip", "Fallout Tokens")],
    },
    UbFranchise {
        slug: "warhammer-40k",
        name: "Warhammer 40,000",
        sets: &[
            ("40k", "Warhammer 40,000 Commander"),
            ("t40k", "Warhammer 40,000 Tokens"),
        ],
    },
    UbFranchise {
        slug: "final-fantasy",
        name: "Final Fantasy",
        sets: &[
            ("fin", "Final Fantasy"),
            ("fic", "Final Fantasy Commander"),
            ("fca", "Final Fantasy: Through the Ages"),
            ("pfin", "Final Fantasy Promos"),
            ("tfin", "Final Fantasy Tokens"),
            ("tfic", "Final Fantasy Commander Tokens"),
            ("afin", "Final Fantasy Art Series"),
            ("afic", "Final Fantasy Scene Box"),
            ("rfin", "Final Fantasy Regional Promos"),
            ("wfin", "FIN Asia WPN Promo Tokens"),
            ("pss5", "FIN Standard Showdown"),
        ],
    },
    UbFranchise {
        slug: "assassins-creed",
        name: "Assassin's Creed",
        sets: &[
            ("acr", "Assassin's Creed"),
            ("tacr", "Assassin's Creed Tokens"),
            ("aacr", "Assassin's Creed Art Series"),
            ("macr", "Assassin's Creed Minigames"),
        ],
    },
    UbFranchise {
        slug: "jurassic-world",
        name: "Jurassic World",
        sets: &[
            ("rex", "Jurassic World Collection"),
            ("trex", "Jurassic World Collection Tokens"),
        ],
    },
    UbFranchise {
        slug: "transformers",
        name: "Transformers",
        sets: &[("bot", "Transformers"), ("tbot", "Transformers Tokens")],
    },
    UbFranchise {
        slug: "marvel",
        name: "Marvel",
        sets: &[
            ("spm", "Marvel's Spider-Man"),
            ("spe", "Marvel's Spider-Man Eternal"),
            ("pspm", "Marvel's Spider-Man Promos"),
            ("aspm", "Marvel's Spider-Man Art Series"),
            ("tspm", "Marvel's Spider-Man Tokens"),
            ("mar", "Marvel Universe"),
            ("lmar", "Marvel Legends Series Inserts"),
            ("msh", "Marvel Super Heroes"),
            ("msc", "Marvel Super Heroes Commander"),
            ("tmsh", "Marvel Super Heroes Tokens"),
            ("tmsc", "Marvel Super Heroes Commander Tokens"),
            ("amsh", "Marvel Super Heroes Art Series"),
            ("fmsc", "Marvel Super Heroes Jumpstart Front Cards"),
        ],
    },
    UbFranchise {
        slug: "avatar-the-last-airbender",
        name: "Avatar: The Last Airbender",
        sets: &[
            ("tla", "Avatar: The Last Airbender"),
            ("tle", "Avatar: The Last Airbender Eternal"),
            ("ptla", "Avatar: The Last Airbender Promos"),
            ("ttla", "Avatar: The Last Airbender Tokens"),
            ("ttle", "Avatar: The Last Airbender Eternal Tokens"),
            ("atla", "Avatar: the Last Airbender Art Series"),
            ("atle", "Avatar: the Last Airbender Eternal Art Series"),
            (
                "ftla",
                "Avatar: The Last Airbender Beginner Box Front Cards",
            ),
            ("jtla", "Avatar: The Last Airbender Jumpstart Front Cards"),
        ],
    },
    UbFranchise {
        slug: "teenage-mutant-ninja-turtles",
        name: "Teenage Mutant Ninja Turtles",
        sets: &[
            ("tmt", "Teenage Mutant Ninja Turtles"),
            ("tmc", "Teenage Mutant Ninja Turtles Eternal"),
            ("pza", "Teenage Mutant Ninja Turtles Source Material"),
            ("ttmt", "Teenage Mutant Ninja Turtles Tokens"),
            ("ttmc", "Teenage Mutant Ninja Turtles Eternal Tokens"),
            ("ftmc", "Teenage Mutant Ninja Turtles Eternal Front Cards"),
            ("atmt", "Teenage Mutant Ninja Turtles Art Series"),
        ],
    },
    UbFranchise {
        slug: "star-trek",
        name: "Star Trek",
        sets: &[
            ("trk", "Star Trek"),
            ("trc", "Star Trek Commander"),
            ("ttrk", "Star Trek Tokens"),
            ("sds", "Stardates"),
        ],
    },
];

/// Every wholly-UB set code across all franchises (the oou_sets overlay and
/// SQL pick predicates).
pub fn all_set_codes() -> impl Iterator<Item = &'static str> {
    FRANCHISES
        .iter()
        .flat_map(|f| f.sets.iter().map(|(code, _)| *code))
}

/// Looks up a franchise by its stored slug.
pub fn franchise_by_slug(slug: &str) -> Option<&'static UbFranchise> {
    FRANCHISES.iter().find(|f| f.slug == slug)
}

/// Set names covered by the given exception slugs, for overlap against
/// `latest_cards.printing_set_names`. Unknown slugs are ignored so a retired
/// franchise slug in stored preferences degrades to no exception.
pub fn exception_set_names(slugs: &[String]) -> Vec<String> {
    slugs
        .iter()
        .filter_map(|s| franchise_by_slug(s))
        .flat_map(|f| f.sets.iter().map(|(_, name)| (*name).to_string()))
        .collect()
}

/// Returns `true` if a printing with this security stamp and set code is out
/// of universe (Universes Beyond).
pub fn is_out_of_universe(security_stamp: Option<&str>, set: &str) -> bool {
    security_stamp == Some("triangle") || all_set_codes().any(|code| code == set)
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
    fn codes_and_slugs_are_lowercase_and_deduplicated() {
        let mut codes = std::collections::HashSet::new();
        for code in all_set_codes() {
            assert_eq!(code, code.to_lowercase(), "set codes are lowercase");
            assert!(codes.insert(code), "duplicate set code: {code}");
        }
        let mut slugs = std::collections::HashSet::new();
        for f in FRANCHISES {
            assert_eq!(f.slug, f.slug.to_lowercase(), "slugs are lowercase");
            assert!(slugs.insert(f.slug), "duplicate franchise slug: {}", f.slug);
            assert!(!f.sets.is_empty(), "franchise {} has no sets", f.slug);
        }
    }

    #[test]
    fn exception_set_names_expands_known_slugs_and_ignores_unknown() {
        let names = exception_set_names(&["doctor-who".to_string(), "not-a-franchise".to_string()]);
        assert_eq!(names, vec!["Doctor Who", "Doctor Who Tokens"]);
        assert!(exception_set_names(&[]).is_empty());
    }
}
