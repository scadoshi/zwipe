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

/// Whether two slug lists pick the same franchises. Order is tap order and
/// means nothing.
pub fn same_set(a: &[String], b: &[String]) -> bool {
    let mut a: Vec<&str> = a.iter().map(String::as_str).collect();
    let mut b: Vec<&str> = b.iter().map(String::as_str).collect();
    a.sort_unstable();
    b.sort_unstable();
    a == b
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

/// `slugs` with `slug` taken out when it is in, else added at the end.
pub fn toggled(slugs: &[String], slug: &str) -> Vec<String> {
    if slugs.iter().any(|s| s == slug) {
        slugs.iter().filter(|s| *s != slug).cloned().collect()
    } else {
        let mut next = slugs.to_vec();
        next.push(slug.to_string());
        next
    }
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
    fn order_does_not_make_a_change() {
        assert!(same_set(&slugs(&["a", "b"]), &slugs(&["b", "a"])));
        assert!(!same_set(&slugs(&["a"]), &slugs(&["a", "b"])));
        assert!(same_set(&[], &[]));
    }

    #[test]
    fn a_tap_adds_at_the_end_or_takes_out() {
        assert_eq!(toggled(&slugs(&["a"]), "b"), slugs(&["a", "b"]));
        assert_eq!(toggled(&slugs(&["a", "b"]), "a"), slugs(&["b"]));
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
