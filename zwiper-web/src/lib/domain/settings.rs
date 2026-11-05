//! Config's rules: optimistic toggles that ignore stale replies, and the
//! Universes Beyond exceptions list.

use zwipe_core::domain::card::scryfall_data::universe;

/// Numbers one toggle's requests, so a reply can tell whether a newer tap
/// has gone out since.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RequestSeq {
    latest: u64,
}

impl RequestSeq {
    /// Numbers a new request, which is now the latest.
    pub fn begin(&mut self) -> u64 {
        self.latest = self.latest.wrapping_add(1);
        self.latest
    }
}

/// What a toggle shows once request `mine` answers: the server's value in
/// `reply`, or `prev` (the value from before that tap) when it failed.
/// `None` leaves the screen alone, since a newer tap's request owns it.
pub fn settle<T>(seq: RequestSeq, mine: u64, reply: Option<T>, prev: T) -> Option<T> {
    (seq.latest == mine).then(|| reply.unwrap_or(prev))
}

/// The saved exceptions as the Config row's chips show them: franchise
/// names, alphabetical, with slugs this build does not know dropped.
pub fn exception_names(slugs: &[String]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = slugs
        .iter()
        .filter_map(|slug| universe::franchise_by_slug(slug))
        .map(|franchise| franchise.name)
        .collect();
    names.sort_unstable();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slugs(items: &[&str]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_reply_overtaken_by_a_newer_tap_changes_nothing() {
        let mut seq = RequestSeq::default();
        let first = seq.begin();
        let second = seq.begin();
        assert_eq!(settle(seq, second, Some(false), true), Some(false));
        assert_eq!(settle(seq, first, Some(true), false), None);
        assert_eq!(settle(seq, first, None, false), None);
    }

    #[test]
    fn the_latest_failure_rolls_back_to_the_value_before_its_tap() {
        let mut seq = RequestSeq::default();
        let _first = seq.begin();
        let second = seq.begin();
        assert_eq!(settle(seq, second, None, true), Some(true));
    }

    #[test]
    fn unknown_slugs_drop_out_of_the_chips() {
        assert_eq!(
            exception_names(&slugs(&["not-a-franchise"])),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn known_slugs_show_by_name_in_order() {
        let mut known: Vec<String> = universe::selectable_franchises()
            .take(3)
            .map(|f| f.slug.to_string())
            .collect();
        known.reverse();
        let names = exception_names(&known);
        assert_eq!(names.len(), known.len());
        assert!(names.is_sorted());
    }
}
