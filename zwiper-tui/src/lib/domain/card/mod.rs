//! A card as text: each face's name, mana cost, type line, rules text and stats.

use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// One face of a card, ready to draw. A single-faced card has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Face {
    pub(crate) name: String,
    pub(crate) mana_cost: Vec<Symbol>,
    pub(crate) type_line: String,
    /// Rules text, one paragraph per entry.
    pub(crate) rules: Vec<String>,
    /// Power and toughness, loyalty or defense, when the face has one.
    pub(crate) stats: Option<String>,
}

/// One mana symbol from a cost such as `{2}{G}{U/P}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Symbol {
    /// The symbol without braces: `2`, `G`, `U/P`.
    pub(crate) text: String,
    /// The first of the five colors it names, in WUBRG order, or `None` for generic and colorless.
    pub(crate) color: Option<usize>,
}

/// The faces of a card, front first.
pub(crate) fn faces(card: &ScryfallData) -> Vec<Face> {
    if let Some(faces) = card.card_faces.as_ref().filter(|f| !f.is_empty()) {
        return faces
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
                    face.defense.as_deref(),
                ),
            })
            .collect();
    }
    vec![Face {
        name: card.name.clone(),
        mana_cost: mana_cost(card.mana_cost.as_deref().unwrap_or_default()),
        type_line: card.type_line.clone().unwrap_or_default(),
        rules: paragraphs(card.oracle_text.as_deref()),
        stats: stats(
            card.power.as_deref(),
            card.toughness.as_deref(),
            card.loyalty.as_deref(),
            card.defense.as_deref(),
        ),
    }]
}

/// Splits a cost like `{2}{G}{U/P}` into its symbols. Text outside braces, such as the ` // ` between split-card halves, is dropped.
pub(crate) fn mana_cost(cost: &str) -> Vec<Symbol> {
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
}
