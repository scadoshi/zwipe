use super::*;
use crate::toast::NORMAL;

fn tag(slug: &str, description: Option<&str>) -> OracleTag {
    OracleTag {
        slug: slug.to_owned(),
        label: slug.replace('-', " "),
        description: description.map(str::to_owned),
        parent_slugs: Vec::new(),
        curated: false,
    }
}

fn catalog() -> Vec<OracleTag> {
    vec![
        tag("ramp", Some("puts lands out")),
        tag("reanimate", None),
        tag("draw", Some("more cards")),
        tag("40k-model", None),
    ]
}

fn slugs(list: &List) -> Vec<String> {
    match list {
        List::Rows(rows) => rows.iter().map(|t| t.slug.clone()).collect(),
        List::Empty(_) => Vec::new(),
    }
}

fn picked(state: &State, tags: &[OracleTag]) -> Vec<char> {
    state
        .rail(tags)
        .into_iter()
        .filter(|chip| chip.picked)
        .map(|chip| chip.letter)
        .collect()
}

#[test]
fn a_new_state_browses_a_with_no_search() {
    let state = State::new();
    assert_eq!(state.query(), "");
    assert_eq!(state.letter(), 'a');
    assert!(!state.searching());
    assert_eq!(state, State::default());
}

#[test]
fn load_offers_the_hint_once_and_ensures_the_catalog_every_time() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Load),
        [
            Effect::OfferHint(OneTimeHint::OracleTagDictionary),
            Effect::EnsureCatalog
        ]
    );
    assert_eq!(state.update(Action::Load), [Effect::EnsureCatalog]);
}

#[test]
fn a_loading_catalog_ghosts() {
    assert_eq!(State::new().body(Catalog::Loading), Body::Loading);
}

#[test]
fn a_failed_catalog_says_so_without_controls() {
    assert_eq!(
        State::new().body(Catalog::Failed),
        Body::Failed(LOAD_FAILED)
    );
}

#[test]
fn every_failure_raises_the_error_toast() {
    let mut state = State::new();
    let toast = Effect::Toast(Toast::new(ToastKind::Error, LOAD_FAILED_TOAST, NORMAL));
    assert_eq!(
        state.update(Action::CatalogFailed),
        std::slice::from_ref(&toast)
    );
    assert_eq!(
        state.update(Action::CatalogFailed),
        [toast],
        "the app raises it each time the catalog lands failed"
    );
}

#[test]
fn a_loaded_catalog_browses_the_letter() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::PickLetter('r'));
    let Body::Browse { rail, list } = state.body(Catalog::Loaded(&tags)) else {
        panic!("a loaded catalog browses");
    };
    assert_eq!(slugs(&list), ["ramp", "reanimate"]);
    assert_eq!(rail.len(), 27, "a digit slug adds #");
    assert_eq!(
        rail.last(),
        Some(&RailLetter {
            letter: dictionary::OTHER,
            label: "#".to_owned(),
            picked: false,
        })
    );
    assert_eq!(picked(&state, &tags), ['r']);
}

#[test]
fn the_rail_labels_letters_uppercase() {
    assert_eq!(letter_label('r'), "R");
    assert_eq!(letter_label(dictionary::OTHER), "#");
    let labels: Vec<String> = State::new()
        .rail(&[tag("ramp", None)])
        .into_iter()
        .map(|chip| chip.label)
        .collect();
    assert_eq!(labels.first().map(String::as_str), Some("A"));
    assert_eq!(labels.last().map(String::as_str), Some("Z"));
}

#[test]
fn an_empty_letter_says_so() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::PickLetter('z'));
    assert_eq!(
        state.list(&tags),
        List::Empty("No tags start with Z.".to_owned())
    );
}

#[test]
fn searching_lists_matches_across_letters_and_unpicks_the_rail() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::PickLetter('r'));
    let effects = state.update(Action::Search("  Cards ".to_owned()));
    assert_eq!(effects, []);
    assert_eq!(state.query(), "  Cards ", "the box keeps what was typed");
    assert!(state.searching());
    assert_eq!(slugs(&state.list(&tags)), ["draw"]);
    assert!(picked(&state, &tags).is_empty(), "no letter draws picked");
}

#[test]
fn a_blank_search_still_browses() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::PickLetter('d'));
    let _ = state.update(Action::Search("   ".to_owned()));
    assert!(!state.searching());
    assert_eq!(slugs(&state.list(&tags)), ["draw"]);
    assert_eq!(picked(&state, &tags), ['d']);
}

#[test]
fn a_search_with_no_match_quotes_it_trimmed_and_lowercased() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::Search(" Zzz ".to_owned()));
    assert_eq!(
        state.list(&tags),
        List::Empty("No tags match \u{201c}zzz\u{201d}.".to_owned())
    );
}

#[test]
fn picking_a_letter_clears_the_search() {
    let tags = catalog();
    let mut state = State::new();
    let _ = state.update(Action::Search("cards".to_owned()));
    let effects = state.update(Action::PickLetter(dictionary::OTHER));
    assert_eq!(effects, []);
    assert_eq!(state.query(), "");
    assert_eq!(state.letter(), dictionary::OTHER);
    assert_eq!(slugs(&state.list(&tags)), ["40k-model"]);
}

#[test]
fn examples_opens_over_the_dictionary() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::OpenExamples("ramp".to_owned())),
        [Effect::Navigate(Destination::Examples("ramp".to_owned()))]
    );
}

#[test]
fn use_hands_the_tag_back_and_keeps_the_dictionary_up() {
    let mut state = State::new();
    let _ = state.update(Action::Search("ramp".to_owned()));
    assert_eq!(
        state.update(Action::Use("ramp".to_owned())),
        [Effect::AdoptTag("ramp".to_owned())],
        "the app's dictionary stays open after Use"
    );
    assert_eq!(state.query(), "ramp", "Use keeps the search");
}

#[test]
fn back_returns_to_the_screen_that_opened_it() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [
            TITLE,
            SEARCH_PLACEHOLDER,
            LOAD_FAILED,
            LOAD_FAILED_TOAST,
            NO_DESCRIPTION,
            EXAMPLES,
            USE,
            BACK
        ],
        [
            "Oracle tags",
            "Search tags or descriptions",
            "Couldn't load the dictionary. Pull back and try again.",
            "Couldn't load the oracle-tag dictionary",
            "No description yet",
            "Examples",
            "Use",
            "Back"
        ]
    );
    assert_eq!(no_match("ramp"), "No tags match \u{201c}ramp\u{201d}.");
    assert_eq!(no_letter('q'), "No tags start with Q.");
    assert_eq!(no_letter(dictionary::OTHER), "No tags start with #.");
}
