//! The lines under a swipe card and in the printing sheet: prices, set, release date and artist, in the app's words.
//!
//! A line that can be missing comes back as `None`. The app draws a missing line as a non-breaking space so the block keeps its height; each client decides how to hold the space.

use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// The price line: `Prices:` then each of the card's USD, EUR and TIX prices it has, in that order, joined by ` |`, such as `Prices: $1.23 | €1.10 | 0.05 TIX`. `None` when the card has none of the three.
///
/// Prices show exactly as Scryfall sends them, with no rounding or reformatting. Foil and etched prices are not shown.
#[must_use]
pub fn price_line(card: &ScryfallData) -> Option<String> {
    let shown = prices(card);
    if shown.is_empty() {
        return None;
    }
    Some(format!("Prices: {}", shown.join(" | ")))
}

/// Each price the card has, one per chip: `$1.23`, then `€1.10`, then
/// `0.05 TIX`, in that order. Empty when it has none of the three.
///
/// Prices show exactly as Scryfall sends them, with no rounding.
#[must_use]
pub fn prices(card: &ScryfallData) -> Vec<String> {
    let prices = &card.prices;
    [
        prices.usd.as_ref().map(|usd| format!("${usd}")),
        prices.eur.as_ref().map(|eur| format!("€{eur}")),
        prices.tix.as_ref().map(|tix| format!("{tix} TIX")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The card's top-level artist, or `None` when it is missing or empty.
#[must_use]
pub fn artist(card: &ScryfallData) -> Option<&str> {
    card.artist.as_deref().filter(|artist| !artist.is_empty())
}

/// The set line: `Set: ` and the set's name.
#[must_use]
pub fn set_line(card: &ScryfallData) -> String {
    format!("Set: {}", card.set_name)
}

/// The release line: `Released: ` and the printing's release date as `YYYY-MM-DD`.
#[must_use]
pub fn released_line(card: &ScryfallData) -> String {
    format!("Released: {}", card.released_at)
}

/// The artist line: `Artist: ` and the card's top-level artist. `None` when the artist is missing or empty.
///
/// Only the top-level artist shows, as in the app, even when a multi-faced card's faces name different artists.
#[must_use]
pub fn artist_line(card: &ScryfallData) -> Option<String> {
    artist(card).map(|artist| format!("Artist: {artist}"))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use zwipe_core::domain::card::scryfall_data::card_faces::{CardFace, CardFaces};

    use super::*;
    use crate::test_support::card;

    fn priced(usd: Option<&str>, eur: Option<&str>, tix: Option<&str>) -> ScryfallData {
        let mut data = card("Bolt").scryfall_data;
        data.prices.usd = usd.map(str::to_owned);
        data.prices.eur = eur.map(str::to_owned);
        data.prices.tix = tix.map(str::to_owned);
        data
    }

    #[test]
    fn prices_lists_each_one_the_card_has_in_order() {
        assert_eq!(
            prices(&priced(Some("1.23"), Some("1.10"), Some("0.05"))),
            ["$1.23", "€1.10", "0.05 TIX"]
        );
        assert_eq!(prices(&priced(None, Some("1.10"), None)), ["€1.10"]);
        assert_eq!(prices(&priced(None, None, None)), Vec::<String>::new());
    }

    #[test]
    fn the_price_line_joins_the_prices() {
        assert_eq!(
            price_line(&priced(Some("1.23"), None, Some("0.05"))).as_deref(),
            Some("Prices: $1.23 | 0.05 TIX")
        );
    }

    #[test]
    fn prices_join_each_currency_the_card_has_in_order() {
        let cases = [
            (
                (Some("1.23"), Some("1.10"), Some("0.05")),
                "Prices: $1.23 | €1.10 | 0.05 TIX",
            ),
            ((Some("1.23"), None, None), "Prices: $1.23"),
            ((None, Some("1.10"), None), "Prices: €1.10"),
            ((None, None, Some("0.05")), "Prices: 0.05 TIX"),
            (
                (Some("1.23"), None, Some("0.05")),
                "Prices: $1.23 | 0.05 TIX",
            ),
        ];
        for ((usd, eur, tix), want) in cases {
            assert_eq!(
                price_line(&priced(usd, eur, tix)).as_deref(),
                Some(want),
                "usd {usd:?} eur {eur:?} tix {tix:?}"
            );
        }
    }

    #[test]
    fn a_card_with_no_prices_has_no_price_line() {
        assert_eq!(price_line(&priced(None, None, None)), None);
    }

    #[test]
    fn foil_and_etched_prices_never_show() {
        let mut data = priced(None, None, None);
        data.prices.usd_foil = Some("9.99".to_owned());
        data.prices.usd_etched = Some("8.88".to_owned());
        data.prices.eur_foil = Some("7.77".to_owned());
        data.prices.eur_etched = Some("6.66".to_owned());
        assert_eq!(price_line(&data), None);
        data.prices.usd = Some("1.00".to_owned());
        assert_eq!(price_line(&data).as_deref(), Some("Prices: $1.00"));
    }

    #[test]
    fn prices_show_verbatim_with_no_rounding() {
        let cases = [
            ("0.00", "Prices: $0.00 | €0.00 | 0.00 TIX"),
            ("12345.67", "Prices: $12345.67 | €12345.67 | 12345.67 TIX"),
            ("0.125", "Prices: $0.125 | €0.125 | 0.125 TIX"),
            ("3", "Prices: $3 | €3 | 3 TIX"),
        ];
        for (price, want) in cases {
            assert_eq!(
                price_line(&priced(Some(price), Some(price), Some(price))).as_deref(),
                Some(want),
                "price {price:?}"
            );
        }
    }

    #[test]
    fn set_and_release_lines_read_as_the_app_shows_them() {
        let mut data = card("Bolt").scryfall_data;
        data.set_name = "Magic 2015".to_owned();
        data.released_at = NaiveDate::from_ymd_opt(2014, 7, 8).unwrap();
        assert_eq!(set_line(&data), "Set: Magic 2015");
        assert_eq!(released_line(&data), "Released: 2014-07-08");
    }

    #[test]
    fn a_missing_or_empty_artist_has_no_artist_line() {
        let mut data = card("Bolt").scryfall_data;
        data.artist = Some("Svetlin Velinov".to_owned());
        assert_eq!(
            artist_line(&data).as_deref(),
            Some("Artist: Svetlin Velinov")
        );
        data.artist = Some(String::new());
        assert_eq!(artist_line(&data), None);
        data.artist = None;
        assert_eq!(artist_line(&data), None);
    }

    #[test]
    fn a_multi_faced_card_shows_only_its_top_level_artist() {
        let face = |name: &str, artist: &str| -> CardFace {
            serde_json::from_value(serde_json::json!({
                "object": "card_face",
                "name": name,
                "mana_cost": "",
                "artist": artist,
            }))
            .unwrap()
        };
        let mut data = card("Front // Back").scryfall_data;
        data.card_faces = Some(CardFaces::from(vec![
            face("Front", "Front Artist"),
            face("Back", "Back Artist"),
        ]));
        data.artist = Some("Front Artist".to_owned());
        assert_eq!(artist_line(&data).as_deref(), Some("Artist: Front Artist"));
        data.artist = None;
        assert_eq!(artist_line(&data), None, "faces are not read");
    }
}
