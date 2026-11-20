//! The checks a Scryfall pull passes before zervice writes any of it.
//!
//! Every check is pure: zervice reads what the database holds, hands it in with what the pull
//! carries, and writes nothing when one refuses. A refusal fails the step, which keeps the stored
//! rows and makes the run exit non-zero, so the failure alert fires.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// The share, in percent, of the stored printings and cards a card pull has to carry.
pub const CARD_FLOOR_PCT: u64 = 90;

/// The share, in percent, of the stored tags and card tags an oracle tags pull has to carry.
pub const TAG_FLOOR_PCT: u64 = 80;

/// The share, in percent, of the profiles carrying a column that a rebuild of it has to leave.
pub const REBUILD_FLOOR_PCT: u64 = 80;

/// The most printings a removal takes out whatever the catalog's size.
pub const REMOVAL_MIN_CAP: u64 = 500;

/// The most printings a removal takes out, in percent of the stored printings, when that is more
/// than [`REMOVAL_MIN_CAP`].
pub const REMOVAL_CAP_PCT: u64 = 1;

/// A bulk file a sync loaded, as `scryfall_loads` records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScryfallLoad {
    /// The bulk file's name.
    pub kind: &'static str,
    /// When Scryfall made it.
    pub updated_at: DateTime<Utc>,
    /// Where it came from.
    pub uri: String,
    /// Items it held.
    pub items: u64,
}

/// A pull, or a rebuild, refused before it changes anything.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefusedPull {
    /// The download is a different size from the one Scryfall lists for the file.
    #[error("the {kind} download is {got} bytes, but Scryfall lists the file at {expected}")]
    Truncated {
        /// The bulk file.
        kind: &'static str,
        /// Bytes downloaded.
        got: u64,
        /// Bytes Scryfall lists.
        expected: u64,
    },
    /// The file is older than the one loaded last.
    #[error("the {kind} file is from {file}, older than the {stored} file loaded last")]
    Stale {
        /// The bulk file.
        kind: &'static str,
        /// When Scryfall made this file.
        file: DateTime<Utc>,
        /// When Scryfall made the file loaded last.
        stored: DateTime<Utc>,
    },
    /// The pull carries none of something the database holds.
    #[error("the pull carries no {what}, so the stored {stored} stay")]
    Empty {
        /// What it counts.
        what: &'static str,
        /// How many the database holds.
        stored: u64,
    },
    /// The pull carries under its floor of something the database holds.
    #[error(
        "the pull carries {pulled} {what}, under {floor_pct}% of the stored {stored}, so the stored rows stay"
    )]
    Shrunk {
        /// What it counts.
        what: &'static str,
        /// How many the pull carries.
        pulled: u64,
        /// How many the database holds.
        stored: u64,
        /// The share it has to carry, in percent.
        floor_pct: u64,
    },
    /// Two of the pull's printings share an id.
    #[error("the card pull lists {count} printing id(s) more than once")]
    DuplicateIds {
        /// Ids past their first.
        count: u64,
    },
    /// Some of the pull's printings have the nil id.
    #[error("the card pull has {count} printing(s) with the nil id")]
    NilIds {
        /// Printings with it.
        count: u64,
    },
    /// The share of printings with a USD price fell under half the stored share.
    #[error(
        "{priced} of the pull's {printings} printings have a USD price, under half the stored share ({stored_priced} of {stored_printings}), so the stored prices stay"
    )]
    PricesWiped {
        /// The pull's printings with a USD price.
        priced: u64,
        /// The pull's printings.
        printings: u64,
        /// Stored printings with a USD price.
        stored_priced: u64,
        /// Stored printings.
        stored_printings: u64,
    },
    /// The removal would take out more printings than its cap.
    #[error(
        "the removal would take out {removing} printings, over its cap of {cap}, so nothing is removed"
    )]
    MassRemoval {
        /// Printings the plan removes.
        removing: u64,
        /// The most it may remove.
        cap: u64,
    },
    /// A rebuild's input tables are empty while the column it writes has data.
    #[error(
        "{what} would rebuild from empty oracle tag tables, so the {stored} profiles carrying it stay"
    )]
    NoInput {
        /// The column rebuilt.
        what: &'static str,
        /// Profiles carrying it now.
        stored: u64,
    },
    /// A rebuild would leave under its floor of the profiles carrying its column.
    #[error(
        "{what} would leave {after} profiles carrying it, under {floor_pct}% of the {before} now, so it rolls back"
    )]
    RebuildShrunk {
        /// The column rebuilt.
        what: &'static str,
        /// Profiles carrying it after the rebuild.
        after: u64,
        /// Profiles carrying it before.
        before: u64,
        /// The share it has to leave, in percent.
        floor_pct: u64,
    },
}

/// Whether `part` is at least `pct` percent of `whole`; anything is at least a share of nothing.
#[must_use]
pub fn at_least(part: u64, whole: u64, pct: u64) -> bool {
    u128::from(part).saturating_mul(100) >= u128::from(whole).saturating_mul(u128::from(pct))
}

/// Accepts a download whose size matches the one Scryfall lists, or any size when it lists none.
///
/// # Errors
///
/// [`RefusedPull::Truncated`] when the sizes differ.
pub fn check_size(kind: &'static str, got: u64, expected: Option<u64>) -> Result<(), RefusedPull> {
    match expected {
        Some(expected) if expected != got => Err(RefusedPull::Truncated {
            kind,
            got,
            expected,
        }),
        Some(_) | None => Ok(()),
    }
}

/// Accepts a file no older than the one loaded last, or any file when none was.
///
/// # Errors
///
/// [`RefusedPull::Stale`] when the file is older.
pub fn check_fresh(
    kind: &'static str,
    file: DateTime<Utc>,
    stored: Option<DateTime<Utc>>,
) -> Result<(), RefusedPull> {
    match stored {
        Some(stored) if file < stored => Err(RefusedPull::Stale { kind, file, stored }),
        Some(_) | None => Ok(()),
    }
}

/// Accepts a pull carrying at least `floor_pct` percent of what the database holds. A pull with
/// none is refused even into an empty database, and any pull into an empty database otherwise
/// passes.
///
/// # Errors
///
/// [`RefusedPull::Empty`] for none, [`RefusedPull::Shrunk`] under the floor.
pub fn check_floor(
    what: &'static str,
    pulled: u64,
    stored: u64,
    floor_pct: u64,
) -> Result<(), RefusedPull> {
    if pulled == 0 {
        return Err(RefusedPull::Empty { what, stored });
    }
    if !at_least(pulled, stored, floor_pct) {
        return Err(RefusedPull::Shrunk {
            what,
            pulled,
            stored,
            floor_pct,
        });
    }
    Ok(())
}

/// What a card catalog holds, stored or pulled.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogCounts {
    /// Printings.
    pub printings: u64,
    /// Distinct cards, by oracle id.
    pub cards: u64,
    /// Printings with a USD price.
    pub priced: u64,
}

impl CatalogCounts {
    /// What `printings` hold.
    #[must_use]
    pub fn of(printings: &[ScryfallData]) -> Self {
        let cards: HashSet<Uuid> = printings.iter().filter_map(|p| p.oracle_id).collect();
        Self {
            printings: count(printings.len()),
            cards: count(cards.len()),
            priced: count(printings.iter().filter(|p| p.prices.usd.is_some()).count()),
        }
    }
}

/// A length as a count.
fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// Accepts a card pull with unique, non-nil printing ids, at least [`CARD_FLOOR_PCT`] percent of
/// the stored printings and cards, and at least half the stored share of USD-priced printings.
///
/// # Errors
///
/// [`RefusedPull::DuplicateIds`], [`RefusedPull::NilIds`], [`RefusedPull::Empty`],
/// [`RefusedPull::Shrunk`] or [`RefusedPull::PricesWiped`], checked in that order.
pub fn check_cards(printings: &[ScryfallData], stored: CatalogCounts) -> Result<(), RefusedPull> {
    let mut seen: HashSet<Uuid> = HashSet::with_capacity(printings.len());
    let duplicates = printings.iter().filter(|p| !seen.insert(p.id)).count();
    if duplicates > 0 {
        return Err(RefusedPull::DuplicateIds {
            count: count(duplicates),
        });
    }
    let nils = printings.iter().filter(|p| p.id.is_nil()).count();
    if nils > 0 {
        return Err(RefusedPull::NilIds { count: count(nils) });
    }
    let pulled = CatalogCounts::of(printings);
    check_floor(
        "printings",
        pulled.printings,
        stored.printings,
        CARD_FLOOR_PCT,
    )?;
    check_floor("cards", pulled.cards, stored.cards, CARD_FLOOR_PCT)?;
    // pulled.priced / pulled.printings < (stored.priced / stored.printings) / 2, cross-multiplied.
    let pulled_share = u128::from(pulled.priced)
        .saturating_mul(u128::from(stored.printings))
        .saturating_mul(2);
    let stored_share = u128::from(stored.priced).saturating_mul(u128::from(pulled.printings));
    if pulled_share < stored_share {
        return Err(RefusedPull::PricesWiped {
            priced: pulled.priced,
            printings: pulled.printings,
            stored_priced: stored.priced,
            stored_printings: stored.printings,
        });
    }
    Ok(())
}

/// The most printings one removal may take out of a catalog of `stored` printings.
#[must_use]
pub fn removal_cap(stored: u64) -> u64 {
    REMOVAL_MIN_CAP.max(stored.saturating_mul(REMOVAL_CAP_PCT) / 100)
}

/// Accepts a removal of up to [`removal_cap`] printings.
///
/// # Errors
///
/// [`RefusedPull::MassRemoval`] over the cap.
pub fn check_removal(removing: u64, stored: u64) -> Result<(), RefusedPull> {
    let cap = removal_cap(stored);
    if removing > cap {
        return Err(RefusedPull::MassRemoval { removing, cap });
    }
    Ok(())
}

/// Accepts a rebuild of `what` unless its input tables are empty while profiles carry it.
///
/// # Errors
///
/// [`RefusedPull::NoInput`] then.
pub fn check_rebuild_input(
    what: &'static str,
    input_rows: u64,
    carrying: u64,
) -> Result<(), RefusedPull> {
    if input_rows == 0 && carrying > 0 {
        return Err(RefusedPull::NoInput {
            what,
            stored: carrying,
        });
    }
    Ok(())
}

/// Accepts a rebuild that leaves at least [`REBUILD_FLOOR_PCT`] percent of the profiles that
/// carried its column.
///
/// # Errors
///
/// [`RefusedPull::RebuildShrunk`] under the floor.
pub fn check_rebuild(what: &'static str, after: u64, before: u64) -> Result<(), RefusedPull> {
    if !at_least(after, before, REBUILD_FLOOR_PCT) {
        return Err(RefusedPull::RebuildShrunk {
            what,
            after,
            before,
            floor_pct: REBUILD_FLOOR_PCT,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 19, hour, 0, 0).unwrap()
    }

    #[test]
    fn a_share_is_measured_exactly_at_its_line() {
        assert!(at_least(90, 100, 90));
        assert!(!at_least(89, 100, 90));
        assert!(at_least(0, 0, 90));
        assert!(at_least(u64::MAX, u64::MAX, 100));
    }

    #[test]
    fn a_download_must_match_the_size_scryfall_lists() {
        assert_eq!(check_size("oracle_tags", 20, Some(20)), Ok(()));
        assert_eq!(check_size("oracle_tags", 20, None), Ok(()));
        assert_eq!(
            check_size("oracle_tags", 19, Some(20)),
            Err(RefusedPull::Truncated {
                kind: "oracle_tags",
                got: 19,
                expected: 20
            })
        );
    }

    #[test]
    fn an_older_file_than_the_last_one_loaded_is_refused() {
        assert_eq!(check_fresh("default_cards", at(22), None), Ok(()));
        assert_eq!(check_fresh("default_cards", at(22), Some(at(22))), Ok(()));
        assert_eq!(check_fresh("default_cards", at(22), Some(at(10))), Ok(()));
        assert!(matches!(
            check_fresh("default_cards", at(10), Some(at(22))),
            Err(RefusedPull::Stale { .. })
        ));
    }

    #[test]
    fn a_pull_must_carry_its_floor_and_never_nothing() {
        assert!(matches!(
            check_floor("tags", 0, 0, TAG_FLOOR_PCT),
            Err(RefusedPull::Empty { .. })
        ));
        assert!(matches!(
            check_floor("tags", 0, 4_606, TAG_FLOOR_PCT),
            Err(RefusedPull::Empty { .. })
        ));
        assert!(matches!(
            check_floor("tags", 3_684, 4_606, TAG_FLOOR_PCT),
            Err(RefusedPull::Shrunk { .. })
        ));
        assert_eq!(check_floor("tags", 3_685, 4_606, TAG_FLOOR_PCT), Ok(()));
        assert_eq!(check_floor("tags", 3, 0, TAG_FLOOR_PCT), Ok(()));
        assert_eq!(check_floor("tags", 9_000, 4_606, TAG_FLOOR_PCT), Ok(()));
    }

    #[test]
    fn a_removal_is_capped_at_500_or_one_percent() {
        assert_eq!(removal_cap(0), 500);
        assert_eq!(removal_cap(50_000), 500);
        assert_eq!(removal_cap(120_211), 1_202);
        assert_eq!(check_removal(1_202, 120_211), Ok(()));
        assert_eq!(
            check_removal(1_203, 120_211),
            Err(RefusedPull::MassRemoval {
                removing: 1_203,
                cap: 1_202
            })
        );
    }

    #[test]
    fn a_rebuild_from_nothing_over_data_is_refused() {
        assert!(matches!(
            check_rebuild_input("card roles", 0, 31_676),
            Err(RefusedPull::NoInput { .. })
        ));
        assert_eq!(check_rebuild_input("card roles", 0, 0), Ok(()));
        assert_eq!(check_rebuild_input("card roles", 241_088, 31_676), Ok(()));
    }

    #[test]
    fn a_rebuild_must_leave_its_floor() {
        assert_eq!(check_rebuild("card roles", 80, 100), Ok(()));
        assert_eq!(check_rebuild("card roles", 5, 0), Ok(()));
        assert!(matches!(
            check_rebuild("card roles", 79, 100),
            Err(RefusedPull::RebuildShrunk { .. })
        ));
    }

    #[test]
    fn a_refusal_names_what_stays() {
        let refused = check_floor("tags", 0, 4_606, TAG_FLOOR_PCT)
            .err()
            .map_or_else(String::new, |e| e.to_string());
        assert_eq!(refused, "the pull carries no tags, so the stored 4606 stay");
    }
}
