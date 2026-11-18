//! Property tests for the collection types an import reads: each statement holds for every
//! input proptest generates, odd cells and junk included.

use proptest::prelude::*;

use super::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionTag, CollectionTags, Finish,
    InvalidCollectionTags, PurchasePrice, is_unseen, strip_unseen,
};

/// `word` in any letter case, with spaces around it.
fn mangled(word: &'static str) -> impl Strategy<Value = String> {
    (
        prop::collection::vec(any::<bool>(), word.len()),
        0usize..3,
        0usize..3,
    )
        .prop_map(move |(upper, before, after)| {
            let cased: String = word
                .chars()
                .zip(upper)
                .map(|(c, up)| if up { c.to_ascii_uppercase() } else { c })
                .collect();
            format!("{}{cased}{}", " ".repeat(before), " ".repeat(after))
        })
}

/// A tag cell: pieces of letters and spaces, some long, some with unseen characters, joined
/// with commas.
fn tag_cell() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-cA-C \u{200b}\u{feff}]{0,44}", 0..26)
        .prop_map(|pieces| pieces.join(","))
}

proptest! {
    /// Guards a row's copies and the merge of two rows of one printing: a quantity exists
    /// exactly for 1 to 9,999, and a merge is the sum held at 9,999 either way round.
    #[test]
    fn a_quantity_is_1_to_9999_and_a_merge_stops_there(
        n in prop_oneof![any::<i32>(), -2..=10_002i32],
        a in 1..=CollectionQuantity::MAX,
        b in 1..=CollectionQuantity::MAX,
    ) {
        prop_assert_eq!(CollectionQuantity::new(n).is_ok(), (1..=CollectionQuantity::MAX).contains(&n));
        let (a, b) = (CollectionQuantity::new(a).unwrap(), CollectionQuantity::new(b).unwrap());
        let want = a.get().checked_add(b.get()).unwrap().min(CollectionQuantity::MAX);
        prop_assert_eq!(a.merged(b).get(), want);
        prop_assert_eq!(a.merged(b), b.merged(a));
    }

    /// Guards the condition cell: every condition's own name, in any case and with spaces,
    /// reads back as itself, and any other text reads as a known condition or the error.
    #[test]
    fn a_condition_reads_back_and_anything_else_never_panics(
        at in 0usize..CardCondition::ALL.len(),
        cased in any::<prop::sample::Index>(),
        junk in any::<String>(),
    ) {
        let condition = CardCondition::ALL[at];
        let words: Vec<String> = condition.as_str().split('_').map(str::to_uppercase).collect();
        let shouted = words.join(" ");
        let spellings = [condition.as_str().to_owned(), shouted];
        let spelling = &spellings[cased.index(spellings.len())];
        prop_assert_eq!(spelling.parse::<CardCondition>(), Ok(condition));
        if let Ok(read) = junk.parse::<CardCondition>() {
            prop_assert_eq!(read.as_str().parse::<CardCondition>(), Ok(read));
        }
    }

    /// Guards the language cell: every language's code, and the older codes and English names
    /// the exports also write, in any case and padded, read back as that language, and any other
    /// text reads as a known language or the error.
    #[test]
    fn a_language_reads_back_and_anything_else_never_panics(
        at in 0usize..CardLanguage::ALL.len(),
        alias in prop::sample::select(vec![
            ("jp", CardLanguage::Ja),
            ("kr", CardLanguage::Ko),
            ("cs", CardLanguage::Zhs),
            ("ct", CardLanguage::Zht),
            ("japanese", CardLanguage::Ja),
            ("german", CardLanguage::De),
            ("chinese simplified", CardLanguage::Zhs),
        ]),
        upper in any::<bool>(),
        pad in 0usize..3,
        junk in any::<String>(),
    ) {
        let (spelling, aliased) = alias;
        let cell = if upper { spelling.to_uppercase() } else { spelling.to_owned() };
        prop_assert_eq!(cell.parse::<CardLanguage>(), Ok(aliased), "{:?}", cell);
        let language = CardLanguage::ALL[at];
        let code = if upper { language.as_str().to_uppercase() } else { language.as_str().to_owned() };
        let cell = format!("{}{code}{}", " ".repeat(pad), " ".repeat(pad));
        prop_assert_eq!(cell.parse::<CardLanguage>(), Ok(language));
        if let Ok(read) = junk.parse::<CardLanguage>() {
            prop_assert_eq!(read.as_str().parse::<CardLanguage>(), Ok(read));
        }
    }

    /// Guards the finish cell: each finish's name in any case and padding reads back as itself.
    #[test]
    fn a_finish_reads_back_and_anything_else_never_panics(
        cell in prop_oneof![mangled("nonfoil"), mangled("foil"), mangled("etched")],
        junk in any::<String>(),
    ) {
        let want = match cell.trim().to_ascii_lowercase().as_str() {
            "foil" => Finish::Foil,
            "etched" => Finish::Etched,
            _ => Finish::Nonfoil,
        };
        prop_assert_eq!(cell.parse::<Finish>(), Ok(want));
        if let Ok(read) = junk.parse::<Finish>() {
            prop_assert_eq!(read.as_str().parse::<Finish>(), Ok(read));
        }
    }

    /// Guards the tag cell: it reads to at most 20 tags, distinct ignoring case, each 1 to 40
    /// visible characters and trimmed, or to the documented error; and read tags written back
    /// with commas read the same.
    #[test]
    fn a_tag_cell_reads_within_its_caps_or_says_why(cell in tag_cell()) {
        let pieces: Vec<String> = cell
            .split(',')
            .map(|p| strip_unseen(p).trim().to_owned())
            .filter(|p| !p.is_empty())
            .collect();
        match CollectionTags::from_cell(&cell) {
            Ok(tags) => {
                prop_assert!(tags.len() <= CollectionTags::MAX);
                let mut seen: Vec<String> = Vec::new();
                for tag in tags.as_slice() {
                    let text = tag.as_str();
                    prop_assert!((1..=CollectionTag::MAX_CHARS).contains(&text.chars().count()), "{text:?}");
                    prop_assert_eq!(text, text.trim());
                    prop_assert!(!text.chars().any(is_unseen), "{text:?}");
                    prop_assert!(!seen.contains(&text.to_lowercase()), "{text:?} twice");
                    seen.push(text.to_lowercase());
                }
                let written: Vec<&str> = tags.as_slice().iter().map(CollectionTag::as_str).collect();
                prop_assert_eq!(CollectionTags::from_cell(&written.join(",")), Ok(tags));
            }
            Err(InvalidCollectionTags::TooLong) => {
                prop_assert!(pieces.iter().any(|p| p.chars().count() > CollectionTag::MAX_CHARS));
            }
            Err(InvalidCollectionTags::TooMany) => {
                let mut distinct: Vec<String> = pieces.iter().map(|p| p.to_lowercase()).collect();
                distinct.sort();
                distinct.dedup();
                prop_assert!(distinct.len() > CollectionTags::MAX);
            }
            Err(InvalidCollectionTags::Empty) => prop_assert!(false, "an empty piece is skipped, never an error"),
        }
    }

    /// Guards the price cell: any price written as either app writes it reads back to the same
    /// cents, and any other text reads as a price in range or the error.
    #[test]
    fn a_price_reads_back_to_the_cent_and_anything_else_never_panics(
        cents in 0..=PurchasePrice::MAX_CENTS,
        style in 0usize..5,
        junk in any::<String>(),
    ) {
        let (whole, part) = (cents.checked_div(100).unwrap(), cents.checked_rem(100).unwrap());
        let cell = match style {
            0 => format!("{whole}.{part:02}"),
            1 => format!("${whole}.{part:02}"),
            2 => format!("{whole}.{part:02} USD"),
            3 => format!("{whole},{part:02}"),
            // One decimal, as `3,5`, when the cents allow it.
            _ if part % 10 == 0 => format!("{whole},{}", part.checked_div(10).unwrap()),
            _ => format!("{whole}.{part:02}"),
        };
        prop_assert_eq!(cell.parse::<PurchasePrice>().map(PurchasePrice::cents), Ok(cents));
        if let Ok(price) = junk.parse::<PurchasePrice>() {
            prop_assert!((0..=PurchasePrice::MAX_CENTS).contains(&price.cents()));
        }
        let negative = format!("-{whole}.{part:02}");
        prop_assert!(negative.parse::<PurchasePrice>().is_err());
    }

    /// Guards every cell's cleaning: control characters, bidi marks and zero-width breaks always
    /// go, it drops exactly the unseen characters, and cleaning twice is cleaning once.
    #[test]
    fn stripping_unseen_characters_drops_exactly_them_once(
        text in any::<String>(),
        hidden in prop::collection::vec(
            prop_oneof![
                prop::char::range('\u{0}', '\u{1f}'),
                prop::char::range('\u{7f}', '\u{9f}'),
                prop::sample::select(vec!['\u{61c}', '\u{200b}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202e}', '\u{2060}', '\u{2066}', '\u{2069}', '\u{feff}']),
            ],
            1..6,
        ),
    ) {
        let marked: String = hidden.iter().copied().chain("Sol Ring".chars()).collect();
        prop_assert_eq!(strip_unseen(&marked), "Sol Ring");
        let clean = strip_unseen(&text);
        prop_assert!(!clean.chars().any(is_unseen));
        let kept: String = text.chars().filter(|c| !is_unseen(*c)).collect();
        prop_assert_eq!(&clean, &kept);
        prop_assert_eq!(strip_unseen(&clean), clean);
    }
}
