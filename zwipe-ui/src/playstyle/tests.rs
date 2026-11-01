use std::collections::BTreeMap;

use zwipe_core::domain::card::scryfall_data::card_faces::{CardFace, CardFaces};
use zwipe_core::http::contracts::suggestions::HttpColorIdentityCount;

use super::*;

/// A commander as the helpers see it: a name and its colors.
type Named = (&'static str, Vec<Color>);

fn commander(name: &'static str, colors: &[Color]) -> Named {
    (name, colors.to_vec())
}

fn colors_of(commander: &Named) -> &[Color] {
    &commander.1
}

fn names(commanders: &[Named]) -> Vec<&str> {
    commanders.iter().map(|c| c.0).collect()
}

use Color::{Black, Blue, Green, Red, White};

#[test]
fn identities_present_are_distinct_fewest_colors_first_colorless_last() {
    let held = [
        commander("Narset", &[White, Blue, Red]),
        commander("Karlov", &[Black, White]),
        commander("Kozilek", &[]),
        commander("Krenko", &[Red]),
        commander("Talrand", &[Blue]),
        commander("Ghired", &[Red, Green, White]),
        commander("Thalia", &[White, Black]),
        commander("Zada", &[Red]),
    ];
    assert_eq!(
        colors::identities_present(held.iter().map(colors_of)),
        vec![
            vec![Blue],
            vec![Red],
            vec![White, Black],
            vec![White, Blue, Red],
            vec![White, Red, Green],
            vec![],
        ]
    );
    assert_eq!(
        colors::identities_present(std::iter::empty()),
        Vec::<Vec<Color>>::new()
    );
    let chosen: &[Color] = &[Green];
    assert_eq!(
        colors::identities_present(held.iter().map(colors_of).chain([chosen])),
        vec![
            vec![Blue],
            vec![Red],
            vec![Green],
            vec![White, Black],
            vec![White, Blue, Red],
            vec![White, Red, Green],
            vec![],
        ],
        "a chosen identity without a commander still gets its chip"
    );
}

#[test]
fn of_identity_keeps_server_order_and_all_passes_everything() {
    let held = [
        commander("Narset", &[White, Blue, Red]),
        commander("Krenko", &[Red]),
        commander("Kozilek", &[]),
        commander("Zada", &[Red]),
        commander("Ghired", &[Red, Green, White]),
    ];
    assert_eq!(
        names(&of_identity(&held, colors_of, Some(&[Red]))),
        vec!["Krenko", "Zada"]
    );
    assert_eq!(
        names(&of_identity(&held, colors_of, Some(&[]))),
        vec!["Kozilek"]
    );
    assert_eq!(
        names(&of_identity(&held, colors_of, Some(&[Red, Green]))),
        Vec::<&str>::new(),
        "a superset identity is not a match"
    );
    assert_eq!(of_identity(&held, colors_of, None).len(), held.len());
}

#[test]
fn shown_is_the_first_ten() {
    let held: Vec<usize> = (0..25).collect();
    let rows = shown(&held);
    assert_eq!(rows.len(), LIST_SIZE);
    assert_eq!(rows.first().copied(), Some(0));
    assert_eq!(rows.last().copied(), Some(9));
    assert_eq!(shown(&[1, 2, 3]).len(), 3, "a short list is shown whole");
}

#[test]
fn empty_identity_line_names_the_identity() {
    assert_eq!(
        empty_identity_line(&[Red, White, Blue]),
        "No Jeskai commanders in your top 100"
    );
    assert_eq!(
        empty_identity_line(&[Blue, Black, Red, Green]),
        "No Glint-Eye commanders in your top 100"
    );
    assert_eq!(
        empty_identity_line(&[Green]),
        "No mono-green commanders in your top 100"
    );
    assert_eq!(
        empty_identity_line(&[]),
        "No colorless commanders in your top 100"
    );
    assert_eq!(
        empty_identity_line(&[White, Blue, Black, Red, Green]),
        "No five-color commanders in your top 100"
    );
}

#[test]
fn a_single_color_is_its_pip_alone_and_others_are_named() {
    assert_eq!(identity_label(&[Blue]), None);
    assert_eq!(identity_label(&[White, Blue, Red]), Some("Jeskai"));
    assert_eq!(identity_label(&[]), Some("Colorless"));
}

fn color(code: &str, decks: u32) -> HttpColorAffinity {
    HttpColorAffinity {
        color: code.to_owned(),
        decks,
    }
}

#[test]
fn color_lines_name_each_color_and_count_decks() {
    let lines = color_lines(&[color("R", 3), color("G", 1)], 4);
    assert_eq!(
        lines,
        vec![
            "You build in red (3 of your 4 decks)",
            "You build in green (1 of your 4 decks)"
        ]
    );
    assert_eq!(
        color_lines(&[color("W", 1)], 1),
        vec!["You build in white (1 of your 1 deck)"]
    );
}

#[test]
fn color_lines_skip_unknown_codes() {
    assert_eq!(color_lines(&[color("C", 2)], 2), Vec::<String>::new());
}

fn tag(slug: &str, role: Option<&str>) -> HttpSharedTag {
    HttpSharedTag {
        slug: slug.to_owned(),
        role: role.map(str::to_owned),
    }
}

fn top(colors: &[&str], decks: u32) -> HttpColorIdentityCount {
    HttpColorIdentityCount {
        colors: colors.iter().map(ToString::to_string).collect(),
        decks,
    }
}

fn profile(
    deck_count: u32,
    colors: Vec<HttpColorAffinity>,
    top: Option<HttpColorIdentityCount>,
) -> HttpBuilderProfile {
    HttpBuilderProfile {
        deck_count,
        colors,
        top_identity: top,
        tags: vec![],
        roles: vec![],
        staples: vec![],
        color_map: None,
    }
}

#[test]
fn staple_percent_rounds_half_up() {
    assert_eq!(staple_percent(2, 3).as_deref(), Some("67%"));
    assert_eq!(staple_percent(1, 3).as_deref(), Some("33%"));
    assert_eq!(
        staple_percent(1, 8).as_deref(),
        Some("13%"),
        "12.5 rounds up"
    );
    assert_eq!(staple_percent(6, 6).as_deref(), Some("100%"));
    assert_eq!(staple_percent(2, 0), None, "no decks to count against");
}

#[test]
fn staple_decks_counts_against_the_total() {
    assert_eq!(staple_decks(11, 13), "11 of 13 decks");
    assert_eq!(staple_decks(1, 1), "1 of 1 deck");
}

/// A staple as the filter sees it: a name, its type line and its color identity.
type Staple = (&'static str, Option<&'static str>, Vec<Color>);

fn staple(name: &'static str, type_line: &'static str, colors: &[Color]) -> Staple {
    (name, Some(type_line), colors.to_vec())
}

fn type_line_of(staple: &Staple) -> Option<&str> {
    staple.1
}

fn identity_of_staple(staple: &Staple) -> &[Color] {
    &staple.2
}

fn staple_names(staples: &[Staple]) -> Vec<&str> {
    staples.iter().map(|s| s.0).collect()
}

fn matching(
    staples: &[Staple],
    card_type: Option<CardType>,
    color: Option<StapleColor>,
) -> Vec<&'static str> {
    matching_staples(staples, type_line_of, identity_of_staple, card_type, color)
        .iter()
        .map(|s| s.0)
        .collect()
}

#[test]
fn a_staples_type_line_falls_back_to_its_first_face() {
    let mut card = crate::test_support::card("Fable of the Mirror-Breaker");
    let face: CardFace = serde_json::from_value(serde_json::json!({
        "object": "card_face",
        "name": "Fable",
        "mana_cost": "{2}{R}",
        "type_line": "Enchantment — Saga",
    }))
    .unwrap();
    card.scryfall_data.card_faces = Some(CardFaces::from(vec![face]));
    let staple = HttpStaple { card, decks: 2 };
    assert_eq!(staple_type_line(&staple), Some("Enchantment — Saga"));
    assert_eq!(
        staple_identity(&staple),
        &*staple.card.scryfall_data.color_identity
    );
}

#[test]
fn a_color_chip_keeps_identities_that_include_it() {
    let staples = [
        staple("Llanowar Elves", "Creature — Elf Druid", &[Green]),
        staple("Sol Ring", "Artifact", &[]),
        staple("Assassin's Trophy", "Instant", &[Black, Green]),
        staple("Swords to Plowshares", "Instant", &[White]),
    ];
    assert_eq!(
        matching(&staples, None, Some(StapleColor::Of(Green))),
        vec!["Llanowar Elves", "Assassin's Trophy"],
        "a multicolor card shows under each of its colors"
    );
    assert_eq!(
        matching(&staples, None, Some(StapleColor::Of(Black))),
        vec!["Assassin's Trophy"]
    );
    assert_eq!(
        matching(&staples, None, Some(StapleColor::Colorless)),
        vec!["Sol Ring"]
    );
    assert_eq!(
        matching(&staples, None, None),
        staple_names(&staples),
        "All on both rows keeps everything"
    );
}

#[test]
fn the_type_and_color_rows_combine() {
    let staples = [
        staple("Llanowar Elves", "Creature — Elf Druid", &[Green]),
        staple("Beast Within", "Instant", &[Green]),
        staple("Solemn Simulacrum", "Artifact Creature — Golem", &[]),
        staple("Thassa's Oracle", "Creature — Merfolk Wizard", &[Blue]),
        staple("Sylvan Library", "Enchantment", &[Green]),
    ];
    assert_eq!(
        matching(
            &staples,
            Some(CardType::Creature),
            Some(StapleColor::Of(Green))
        ),
        vec!["Llanowar Elves"]
    );
    assert_eq!(
        matching(&staples, Some(CardType::Creature), None),
        vec!["Llanowar Elves", "Solemn Simulacrum", "Thassa's Oracle"]
    );
    assert_eq!(
        matching(&staples, Some(CardType::Artifact), None),
        vec!["Solemn Simulacrum"],
        "an artifact creature counts as an artifact too"
    );
    assert!(
        matching(
            &staples,
            Some(CardType::Instant),
            Some(StapleColor::Of(Blue))
        )
        .is_empty(),
        "no blue instant, so nothing matches"
    );
}

#[test]
fn matching_staples_keeps_order_and_cuts_to_ten() {
    let names: [&'static str; 12] = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"];
    let mut staples: Vec<Staple> = names
        .iter()
        .map(|name| staple(name, "Creature", &[Red]))
        .collect();
    staples.insert(3, staple("Sol Ring", "Artifact", &[]));
    assert_eq!(
        matching(&staples, Some(CardType::Creature), None),
        names.iter().take(LIST_SIZE).copied().collect::<Vec<_>>(),
        "the first ten matches in the order sent"
    );
    assert_eq!(matching(&staples, None, None).len(), LIST_SIZE);
}

#[test]
fn only_matched_chips_are_listed_in_fixed_order() {
    let staples = [
        staple("Command Tower", "Land", &[]),
        staple("Beast Within", "Instant", &[Green]),
        staple("Solemn Simulacrum", "Artifact Creature — Golem", &[]),
        staple("Lightning Helix", "Instant", &[Red, White]),
    ];
    assert_eq!(
        staple_types_present(&staples, type_line_of),
        vec![
            CardType::Creature,
            CardType::Artifact,
            CardType::Instant,
            CardType::Land
        ]
    );
    assert_eq!(
        staple_colors_present(&staples, identity_of_staple),
        vec![
            StapleColor::Of(White),
            StapleColor::Of(Red),
            StapleColor::Of(Green),
            StapleColor::Colorless
        ]
    );
    let colored = [staple("Beast Within", "Instant", &[Green])];
    assert_eq!(
        staple_colors_present(&colored, identity_of_staple),
        vec![StapleColor::Of(Green)],
        "no colorless chip without a colorless staple"
    );
    assert_eq!(
        staple_types_present::<Staple>(&[], type_line_of),
        Vec::<CardType>::new()
    );
    assert_eq!(StapleColor::Of(Red).pips(), vec![Red]);
    assert_eq!(StapleColor::Colorless.pips(), Vec::<Color>::new());
}

fn value(pips: &[Color], count: &str) -> SummaryValue {
    SummaryValue {
        pips: pips.to_vec(),
        count: count.to_owned(),
    }
}

#[test]
fn profile_summary_counts_decks_with_the_top_set_and_color() {
    let summary = profile_summary(&profile(
        7,
        vec![color("U", 5), color("W", 4)],
        Some(top(&["R", "W", "U"], 4)),
    ));
    assert_eq!(summary.decks, "7");
    assert_eq!(
        summary.color_set,
        Some(value(&[White, Blue, Red], "(4)")),
        "pips in WUBRG order whatever order the server sent"
    );
    assert_eq!(summary.color, Some(value(&[Blue], "(5)")));

    let summary = profile_summary(&profile(1, vec![color("B", 1)], Some(top(&["B"], 1))));
    assert_eq!(summary.decks, "1");
    assert_eq!(summary.color_set, Some(value(&[Black], "(1)")));
    assert_eq!(summary.color, Some(value(&[Black], "(1)")));
}

#[test]
fn profile_summary_omits_what_the_profile_lacks() {
    let summary = profile_summary(&profile(3, vec![color("U", 2)], None));
    assert_eq!(summary.color_set, None, "no identity repeats");
    assert_eq!(summary.color, Some(value(&[Blue], "(2)")));

    let summary = profile_summary(&profile(2, vec![], Some(top(&[], 2))));
    assert_eq!(summary.color_set, Some(value(&[], "(2)")));
    assert_eq!(summary.color, None, "every deck colorless");

    let summary = profile_summary(&profile(2, vec![color("C", 2), color("G", 1)], None));
    assert_eq!(
        summary.color,
        Some(value(&[Green], "(1)")),
        "a code outside WUBRG is skipped"
    );
}

fn profile_tag(
    slug: &str,
    role: Option<&str>,
    decks: u32,
    cards: u32,
    score: f64,
) -> HttpProfileTag {
    HttpProfileTag {
        slug: slug.to_owned(),
        role: role.map(str::to_owned),
        decks,
        cards,
        score,
    }
}

fn bar(key: &str, label: &str, count: u64, pct: u32) -> Bar {
    Bar {
        key: key.to_owned(),
        label: label.to_owned(),
        count,
        pct,
        revealable: true,
    }
}

#[test]
fn a_color_map_has_data_with_any_colored_or_colorless_card() {
    assert!(!color_map_has_data(&ColorMapCounts::default()));
    assert!(color_map_has_data(&ColorMapCounts {
        colorless: 1,
        ..ColorMapCounts::default()
    }));
    assert!(color_map_has_data(&ColorMapCounts {
        cards: [0, 0, 0, 0, 2],
        ..ColorMapCounts::default()
    }));
}

#[test]
fn a_color_map_reads_only_wubrg_codes_for_data() {
    use zwipe_core::http::contracts::suggestions::{HttpColorCards, HttpColorMap};

    let map = |code: &str| HttpColorMap {
        colors: vec![HttpColorCards {
            color: code.to_owned(),
            cards: 3,
        }],
        colorless_cards: 0,
        pairs: Vec::new(),
    };
    assert!(color_map_has_data(&ColorMapCounts::from(&map("G"))));
    assert!(
        !color_map_has_data(&ColorMapCounts::from(&map("C"))),
        "a code outside WUBRG is no data, as in the app"
    );
}

#[test]
fn role_bars_keep_the_order_sent_and_abbreviate_like_the_deck_chart() {
    let roles = [
        HttpProfileRole {
            role: "sacrifice".to_owned(),
            decks: 50,
        },
        HttpProfileRole {
            role: "ramp".to_owned(),
            decks: 1,
        },
    ];
    let tags = [
        profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
        profile_tag("treasure", Some("ramp"), 1, 4, 0.2),
    ];
    assert_eq!(
        role_bars(&roles, &tags),
        vec![
            bar("sacrifice", "sacrf", 50, 100),
            bar("ramp", "ramp", 1, 4)
        ],
        "a nonzero bar is at least the 4% sliver"
    );
    assert_eq!(role_bars(&[], &tags), Vec::<Bar>::new());
}

#[test]
fn a_role_with_no_tags_sent_is_not_revealable() {
    let roles = [
        HttpProfileRole {
            role: "sacrifice".to_owned(),
            decks: 2,
        },
        HttpProfileRole {
            role: "ramp".to_owned(),
            decks: 1,
        },
    ];
    let tags = [profile_tag("aristocrats", Some("sacrifice"), 2, 5, 0.4)];
    let revealable: Vec<(String, bool)> = role_bars(&roles, &tags)
        .into_iter()
        .map(|b| (b.key, b.revealable))
        .collect();
    assert_eq!(
        revealable,
        vec![("sacrifice".to_owned(), true), ("ramp".to_owned(), false)]
    );
}

#[test]
fn tag_rows_show_the_top_ten_while_roles_read_every_tag_sent() {
    let mut tags: Vec<HttpProfileTag> = (0..20)
        .map(|i| profile_tag(&format!("removal-{i:02}"), Some("removal"), 1, 1, 1.0))
        .collect();
    tags.push(profile_tag("mana-rock", Some("ramp"), 1, 1, 0.5));
    let rows = tag_rows(&tags);
    assert_eq!(rows.len(), TAG_TABLE_ROWS);
    assert!(rows.iter().all(|r| r.slug.starts_with("removal-")));
    assert_eq!(role_tags(&tags, "ramp"), vec!["mana-rock"]);
}

#[test]
fn tag_rows_keep_the_order_sent_and_scale_impact_to_the_top_tag() {
    let tags = [
        profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
        profile_tag("treasure", Some("ramp"), 3, 40, 0.8),
        profile_tag("aristocrats", Some("sacrifice"), 3, 9, 0.4),
        profile_tag("typal-goblin", None, 1, 2, 0.01),
        profile_tag("blink", None, 1, 1, 0.0),
    ];
    let rows = tag_rows(&tags);
    let got: Vec<(&str, u32, u32, Option<u32>)> = rows
        .iter()
        .map(|r| (r.slug.as_str(), r.decks, r.cards, r.impact))
        .collect();
    assert_eq!(
        got,
        vec![
            ("sacrifice-outlet", 6, 23, Some(100)),
            ("treasure", 3, 40, Some(50)),
            ("aristocrats", 3, 9, Some(25)),
            ("typal-goblin", 1, 2, Some(1)),
            ("blink", 1, 1, Some(0)),
        ],
        "the order sent, impact rounded against the top score"
    );
    assert_eq!(tag_rows(&[]), Vec::<TagRow>::new());
}

#[test]
fn tag_rows_round_impact_to_the_nearest_whole() {
    let rows = tag_rows(&[
        profile_tag("a", None, 1, 1, 3.0),
        profile_tag("b", None, 1, 1, 2.0),
        profile_tag("c", None, 1, 1, 1.0),
    ]);
    let got: Vec<Option<u32>> = rows.iter().map(|r| r.impact).collect();
    assert_eq!(got, vec![Some(100), Some(67), Some(33)]);
}

#[test]
fn tag_rows_with_no_score_show_no_impact() {
    let rows = tag_rows(&[
        profile_tag("treasure", None, 2, 5, 0.0),
        profile_tag("blink", None, 1, 3, 0.0),
    ]);
    let got: Vec<(u32, u32, Option<u32>)> =
        rows.iter().map(|r| (r.decks, r.cards, r.impact)).collect();
    assert_eq!(
        got,
        vec![(2, 5, None), (1, 3, None)],
        "an older server's rows keep their counts and leave impact empty"
    );
}

#[test]
fn impact_reads_nothing_against_no_score_and_zero_below_it() {
    assert_eq!(impact(1.0, 0.0), None);
    assert_eq!(impact(1.0, f64::NAN), None);
    assert_eq!(impact(-3.0, 2.0), Some(0));
    assert_eq!(impact(f64::NAN, 2.0), Some(0));
    assert_eq!(impact(5.0, 2.0), Some(TOP_IMPACT), "clamped to the top");
}

#[test]
fn role_tags_keep_the_order_sent() {
    let tags = [
        profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
        profile_tag("treasure", Some("ramp"), 3, 40, 0.8),
        profile_tag("aristocrats", Some("sacrifice"), 3, 9, 0.4),
        profile_tag("typal-goblin", None, 1, 2, 0.01),
    ];
    assert_eq!(
        role_tags(&tags, "sacrifice"),
        vec!["sacrifice-outlet", "aristocrats"]
    );
    assert_eq!(role_tags(&tags, "draw"), Vec::<String>::new());
}

#[test]
fn shared_tags_group_by_role_in_first_appearance_order() {
    let grouped = group_shared_tags(&[
        tag("sacrifice-outlet", Some("sacrifice")),
        tag("typal-goblin", None),
        tag("spot-removal", Some("removal")),
        tag("aristocrats", Some("sacrifice")),
    ]);
    assert_eq!(grouped.roles, vec!["sacrifice", "removal"]);
    let expected: BTreeMap<String, Vec<String>> = BTreeMap::from([
        (
            "sacrifice".to_owned(),
            vec!["sacrifice-outlet".to_owned(), "aristocrats".to_owned()],
        ),
        ("removal".to_owned(), vec!["spot-removal".to_owned()]),
    ]);
    assert_eq!(grouped.by_role, expected);
    assert_eq!(grouped.other, vec!["typal-goblin"]);
}
