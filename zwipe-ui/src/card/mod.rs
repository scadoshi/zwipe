//! A card as text: each face's name, mana cost, type line, rules text and stats.

use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// One face of a card, ready to draw. A single-faced card has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
    /// The face's name.
    pub name: String,
    /// The face's mana cost as symbols.
    pub mana_cost: Vec<Symbol>,
    /// The face's type line, empty when it has none.
    pub type_line: String,
    /// Rules text, one paragraph per entry.
    pub rules: Vec<String>,
    /// Power and toughness, loyalty or defense, when the face has one.
    pub stats: Option<String>,
}

/// One mana symbol from a cost such as `{2}{G}{U/P}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// The symbol without braces: `2`, `G`, `U/P`.
    pub text: String,
    /// The first of the five colors it names, in WUBRG order, or `None` for generic and colorless.
    pub color: Option<usize>,
}

/// The faces a card's details show, front first, as the app's card details
/// choose them.
///
/// A card with no top-level rules text and faces shows each face that has
/// rules text or a type line; a face's stats never include defense, as in
/// the app. Any other card shows its top-level text as one face. A card
/// with neither rules text nor a type line has nothing to show and gives no
/// faces.
#[must_use]
pub fn faces(card: &ScryfallData) -> Vec<Face> {
    if card.oracle_text.is_none()
        && let Some(faces) = card.card_faces.as_ref()
    {
        let per_face: Vec<Face> = faces
            .iter()
            .map(|face| Face {
                name: face.name.clone(),
                mana_cost: mana_cost(&face.mana_cost),
                type_line: face.type_line.clone().unwrap_or_default(),
                rules: paragraphs(face.oracle_text.as_deref()),
                stats: stats(
                    face.power.as_deref(),
                    face.toughness.as_deref(),
                    face.loyalty.as_deref(),
                    None,
                ),
            })
            .filter(|face| !face.rules.is_empty() || !face.type_line.is_empty())
            .collect();
        if !per_face.is_empty() {
            return per_face;
        }
    }
    let rules = paragraphs(card.oracle_text.as_deref());
    let type_line = card.type_line.clone().unwrap_or_default();
    if rules.is_empty() && type_line.is_empty() {
        return Vec::new();
    }
    vec![Face {
        name: card.name.clone(),
        mana_cost: mana_cost(card.mana_cost.as_deref().unwrap_or_default()),
        type_line,
        rules,
        stats: stats(
            card.power.as_deref(),
            card.toughness.as_deref(),
            card.loyalty.as_deref(),
            card.defense.as_deref(),
        ),
    }]
}

/// How many faces a card's details show behind Flip: 1 for most cards, more
/// for a multi-faced card, 0 when there is nothing to show.
#[must_use]
pub fn face_count(card: &ScryfallData) -> usize {
    faces(card).len()
}

/// Splits a cost like `{2}{G}{U/P}` into its symbols. Text outside braces, such as the ` // ` between split-card halves, is dropped.
#[must_use]
pub fn mana_cost(cost: &str) -> Vec<Symbol> {
    cost.split('{')
        .filter_map(|part| part.split_once('}').map(|(symbol, _)| symbol))
        .filter(|symbol| !symbol.is_empty())
        .map(|symbol| Symbol {
            text: symbol.to_owned(),
            color: symbol.chars().find_map(color_index),
        })
        .collect()
}

fn color_index(c: char) -> Option<usize> {
    "WUBRG".find(c)
}

fn paragraphs(text: Option<&str>) -> Vec<String> {
    text.map(|t| t.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

fn stats(
    power: Option<&str>,
    toughness: Option<&str>,
    loyalty: Option<&str>,
    defense: Option<&str>,
) -> Option<String> {
    match (power, toughness) {
        (Some(p), Some(t)) => Some(format!("{p}/{t}")),
        _ => loyalty
            .map(|l| format!("Loyalty {l}"))
            .or_else(|| defense.map(|d| format!("Defense {d}"))),
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::scryfall_data::card_faces::{CardFace, CardFaces};

    use super::*;
    use crate::test_support::card;

    fn symbols(cost: &str) -> Vec<(String, Option<usize>)> {
        mana_cost(cost)
            .into_iter()
            .map(|s| (s.text, s.color))
            .collect()
    }

    #[test]
    fn mana_cost_reads_generic_colored_and_hybrid_symbols() {
        assert_eq!(
            symbols("{2}{G}{U/P}{W/B}"),
            [
                ("2".to_owned(), None),
                ("G".to_owned(), Some(4)),
                ("U/P".to_owned(), Some(1)),
                ("W/B".to_owned(), Some(0)),
            ]
        );
        assert_eq!(
            symbols("{C}{X}"),
            [("C".to_owned(), None), ("X".to_owned(), None)]
        );
        assert_eq!(symbols(""), []);
        assert_eq!(symbols("{R} // {U}").len(), 2);
    }

    #[test]
    fn a_creature_shows_its_cost_type_rules_and_stats() {
        let mut data = card("Llanowar Elves").scryfall_data;
        data.mana_cost = Some("{G}".to_owned());
        data.type_line = Some("Creature — Elf Druid".to_owned());
        data.oracle_text = Some("{T}: Add {G}.".to_owned());
        data.power = Some("1".to_owned());
        data.toughness = Some("1".to_owned());

        let faces = faces(&data);
        let face = faces.first().unwrap();
        assert_eq!(face.name, "Llanowar Elves");
        assert_eq!(face.mana_cost.len(), 1);
        assert_eq!(face.type_line, "Creature — Elf Druid");
        assert_eq!(face.rules, ["{T}: Add {G}."]);
        assert_eq!(face.stats.as_deref(), Some("1/1"));
    }

    #[test]
    fn a_planeswalker_shows_loyalty_and_rules_keep_their_paragraphs() {
        let mut data = card("Walker").scryfall_data;
        data.loyalty = Some("4".to_owned());
        data.oracle_text = Some("+1: Draw a card.\n−3: Deal 3 damage.".to_owned());
        let face = faces(&data).into_iter().next().unwrap();
        assert_eq!(face.stats.as_deref(), Some("Loyalty 4"));
        assert_eq!(face.rules.len(), 2);
    }

    #[test]
    fn a_double_faced_card_shows_both_faces_front_first() {
        let mut data = card("Front // Back").scryfall_data;
        let face = |name: &str, cost: &str| -> CardFace {
            serde_json::from_value(serde_json::json!({
                "object": "card_face",
                "name": name,
                "mana_cost": cost,
                "type_line": "Creature",
                "power": "2",
                "toughness": "2",
            }))
            .unwrap()
        };
        data.card_faces = Some(CardFaces::from(vec![
            face("Front", "{1}{R}"),
            face("Back", ""),
        ]));
        let faces = faces(&data);
        assert_eq!(
            faces.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            ["Front", "Back"]
        );
        assert_eq!(faces.get(1).map(|f| f.mana_cost.len()), Some(0));
    }

    fn face(name: &str, type_line: Option<&str>, text: Option<&str>) -> CardFace {
        serde_json::from_value(serde_json::json!({
            "object": "card_face",
            "name": name,
            "mana_cost": "",
            "type_line": type_line,
            "oracle_text": text,
            "defense": "5",
        }))
        .unwrap()
    }

    #[test]
    fn faces_are_read_only_when_the_card_has_no_top_level_rules() {
        let mut data = card("Split").scryfall_data;
        data.oracle_text = Some("Whole card text.".to_owned());
        data.type_line = Some("Instant // Instant".to_owned());
        data.card_faces = Some(CardFaces::from(vec![
            face("Left", Some("Instant"), Some("Left text.")),
            face("Right", Some("Instant"), Some("Right text.")),
        ]));
        let shown = faces(&data);
        assert_eq!(shown.len(), 1, "top-level text wins, as in the app");
        assert_eq!(shown[0].rules, ["Whole card text."]);
    }

    #[test]
    fn faces_with_no_text_or_type_drop_and_never_show_defense() {
        let mut data = card("Battle // Back").scryfall_data;
        data.card_faces = Some(CardFaces::from(vec![
            face("Battle", Some("Battle — Siege"), Some("Front text.")),
            face("Blank", None, None),
        ]));
        let shown = faces(&data);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].name, "Battle");
        assert_eq!(shown[0].stats, None, "the app passes no defense for a face");
        assert_eq!(face_count(&data), 1);
    }

    #[test]
    fn a_card_with_no_rules_or_type_line_has_nothing_to_show() {
        assert_eq!(face_count(&card("Blank").scryfall_data), 0);
    }
}
