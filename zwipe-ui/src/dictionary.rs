//! The oracle tag dictionary's listing: which letter a tag files under, the
//! letters the rail offers, and the rows on screen for a search or a letter.
//!
//! Every client lists the same tags in the same order for the same search,
//! as the app does it.

use zwipe_core::domain::card::oracle_tag::{self, OracleTag};

use crate::timing::DICTIONARY_RESULT_CAP;

/// The bucket for slugs that start with anything but an ASCII letter:
/// digits, symbols, accented letters and the empty slug.
pub const OTHER: char = '#';

/// The letter a slug files under: its first character lowercased when it is
/// an ASCII letter, else [`OTHER`].
#[must_use]
pub fn bucket_of(slug: &str) -> char {
    match slug.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_lowercase(),
        _ => OTHER,
    }
}

/// The letter rail: `a` to `z`, then [`OTHER`] only when some tag files
/// under it.
#[must_use]
pub fn letters(tags: &[OracleTag]) -> Vec<char> {
    let other = tags.iter().any(|t| bucket_of(&t.slug) == OTHER);
    ('a'..='z').chain(other.then_some(OTHER)).collect()
}

/// The rows the dictionary lists. A query that is blank after trimming
/// browses: the tags under `letter`, in slug order. Any other query is the
/// shared ranked catalog search ([`oracle_tag::search_oracle_tags`]), which
/// ignores case and `letter`, cut to [`DICTIONARY_RESULT_CAP`].
#[must_use]
pub fn rows_for(tags: &[OracleTag], query: &str, letter: char) -> Vec<OracleTag> {
    if query.trim().is_empty() {
        let mut in_letter: Vec<OracleTag> = tags
            .iter()
            .filter(|t| bucket_of(&t.slug) == letter)
            .cloned()
            .collect();
        in_letter.sort_by(|a, b| a.slug.cmp(&b.slug));
        in_letter
    } else {
        let mut matches = oracle_tag::search_oracle_tags(tags, query);
        matches.truncate(DICTIONARY_RESULT_CAP);
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(slug: &str, description: Option<&str>) -> OracleTag {
        OracleTag {
            slug: slug.to_owned(),
            label: slug.replace('-', " "),
            description: description.map(str::to_owned),
            parent_slugs: Vec::new(),
            curated: false,
        }
    }

    fn slugs(rows: Vec<OracleTag>) -> Vec<String> {
        rows.into_iter().map(|t| t.slug).collect()
    }

    #[test]
    fn a_letter_files_under_itself_lowercased() {
        assert_eq!(bucket_of("ramp"), 'r');
        assert_eq!(bucket_of("Ramp"), 'r');
        assert_eq!(bucket_of("z"), 'z');
    }

    #[test]
    fn anything_but_an_ascii_letter_files_under_other() {
        for slug in [
            "40k-model",
            "5c-matters",
            "",
            "-dash",
            "+1-counters",
            "éclair",
            " ramp",
        ] {
            assert_eq!(bucket_of(slug), OTHER, "slug {slug:?}");
        }
    }

    #[test]
    fn the_rail_adds_other_only_when_a_tag_needs_it() {
        let plain = [tag("ramp", None)];
        let rail = letters(&plain);
        assert_eq!(rail.len(), 26);
        assert_eq!(rail.first(), Some(&'a'));
        assert_eq!(rail.last(), Some(&'z'));

        let with_digit = [tag("ramp", None), tag("40k-model", None)];
        let rail = letters(&with_digit);
        assert_eq!(rail.len(), 27);
        assert_eq!(rail.last(), Some(&OTHER));

        assert_eq!(letters(&[]).len(), 26);
    }

    #[test]
    fn browsing_lists_one_letter_in_slug_order() {
        let tags = [
            tag("reanimate", None),
            tag("draw", None),
            tag("Ramp-big", None),
            tag("ramp", None),
        ];
        assert_eq!(
            slugs(rows_for(&tags, "", 'r')),
            ["Ramp-big", "ramp", "reanimate"]
        );
    }

    #[test]
    fn browsing_other_lists_digits_and_symbols() {
        let tags = [tag("ramp", None), tag("5c", None), tag("40k-model", None)];
        assert_eq!(slugs(rows_for(&tags, "", OTHER)), ["40k-model", "5c"]);
    }

    #[test]
    fn a_blank_query_browses() {
        let tags = [tag("ramp", None), tag("draw", None)];
        assert_eq!(slugs(rows_for(&tags, "   ", 'd')), ["draw"]);
    }

    #[test]
    fn searching_reads_descriptions_across_letters() {
        let tags = [
            tag("ramp", Some("puts lands out")),
            tag("draw", Some("more cards")),
        ];
        assert_eq!(slugs(rows_for(&tags, "cards", 'r')), ["draw"]);
    }

    #[test]
    fn searching_ranks_exact_then_name_then_description() {
        let tags = [
            tag("wipe", Some("removal for every creature")),
            tag("removal-artifact", None),
            tag("removal", None),
            tag("artifact-removal", None),
        ];
        assert_eq!(
            slugs(rows_for(&tags, "Removal", 'a')),
            ["removal", "artifact-removal", "removal-artifact", "wipe"]
        );
    }

    #[test]
    fn searching_stops_at_the_cap() {
        let tags: Vec<OracleTag> = (0..DICTIONARY_RESULT_CAP + 20)
            .map(|i| tag(&format!("ramp-{i:03}"), None))
            .collect();
        let rows = rows_for(&tags, "ramp", 'r');
        assert_eq!(rows.len(), DICTIONARY_RESULT_CAP);
        assert_eq!(rows.first().map(|t| t.slug.as_str()), Some("ramp-000"));
    }
}
