//! Which printing Quick add lands: a card's cheapest regular printing, not its newest.
//!
//! Regular means what import lands for a line that names no printing (English, paper, regular
//! size, not a promo, sold in nonfoil, never an Art Series, token or memorabilia printing) and no
//! special treatment on top: no borderless, full-art, textless, showcase, extended-art, etched or
//! inverted frame. Cheapest is by the USD price; a tie goes to the newest.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

/// Layouts that are no playable printing of the card.
const NOT_A_PRINTING: [&str; 4] = ["art_series", "token", "double_faced_token", "emblem"];

/// Set types that are no playable printing of the card.
const NOT_A_SET: [&str; 2] = ["memorabilia", "token"];

/// Frame effects that make a printing a special treatment.
const SPECIAL_FRAMES: [&str; 5] = ["showcase", "extendedart", "etched", "inverted", "fullart"];

/// Whether `printing` is a regular one, as the module says.
#[must_use]
pub fn is_regular(printing: &ScryfallData) -> bool {
    let special_frame = printing
        .frame_effects
        .as_ref()
        .is_some_and(|effects| effects.iter().any(|e| SPECIAL_FRAMES.contains(&e.as_str())));
    printing.lang == "en"
        && !printing.digital
        && !printing.oversized
        && !printing.promo
        && printing.finishes.iter().any(|f| f == "nonfoil")
        && !NOT_A_PRINTING.contains(&printing.layout.as_str())
        && !NOT_A_SET.contains(&printing.set_type.as_str())
        && printing.border_color != "borderless"
        && !printing.full_art
        && !printing.textless
        && !special_frame
}

/// The USD price of `printing`, when it has one that reads as a number.
fn usd(printing: &ScryfallData) -> Option<f64> {
    printing.prices.usd.as_deref()?.parse::<f64>().ok()
}

/// The cheapest regular printing among `printings`, the newest of any tied; `None` when no
/// regular printing has a USD price, and the caller keeps the printing it has.
#[must_use]
pub fn cheapest_regular(printings: &[Card]) -> Option<&Card> {
    printings
        .iter()
        .filter(|card| is_regular(&card.scryfall_data))
        .filter_map(|card| usd(&card.scryfall_data).map(|price| (price, card)))
        .min_by(|(a, x), (b, y)| {
            a.total_cmp(b).then(
                y.scryfall_data
                    .released_at
                    .cmp(&x.scryfall_data.released_at),
            )
        })
        .map(|(_, card)| card)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::test_support::card;

    fn printing(price: Option<&str>, released: (i32, u32, u32)) -> Card {
        let mut c = card("Sol Ring");
        c.scryfall_data.lang = "en".to_owned();
        c.scryfall_data.finishes = vec!["nonfoil".to_owned(), "foil".to_owned()];
        c.scryfall_data.layout = "normal".to_owned();
        c.scryfall_data.set_type = "expansion".to_owned();
        c.scryfall_data.border_color = "black".to_owned();
        c.scryfall_data.prices.usd = price.map(str::to_owned);
        let (y, m, d) = released;
        c.scryfall_data.released_at = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        c
    }

    #[test]
    fn the_cheapest_regular_printing_wins_over_the_newest() {
        let newest = printing(Some("3.50"), (2026, 6, 1));
        let cheap = printing(Some("0.75"), (2019, 1, 1));
        let middle = printing(Some("1.25"), (2022, 1, 1));
        let printings = [newest, cheap.clone(), middle];
        assert_eq!(
            cheapest_regular(&printings).map(|c| c.scryfall_data.id),
            Some(cheap.scryfall_data.id)
        );
    }

    #[test]
    fn a_tie_goes_to_the_newest() {
        let old = printing(Some("1.00"), (2015, 1, 1));
        let new = printing(Some("1.00"), (2024, 1, 1));
        let printings = [old, new.clone()];
        assert_eq!(
            cheapest_regular(&printings).map(|c| c.scryfall_data.id),
            Some(new.scryfall_data.id)
        );
    }

    #[test]
    fn promos_and_special_treatments_never_win_however_cheap() {
        let regular = printing(Some("2.00"), (2020, 1, 1));
        let mut promo = printing(Some("0.10"), (2021, 1, 1));
        promo.scryfall_data.promo = true;
        let mut borderless = printing(Some("0.10"), (2021, 1, 1));
        borderless.scryfall_data.border_color = "borderless".to_owned();
        let mut showcase = printing(Some("0.10"), (2021, 1, 1));
        showcase.scryfall_data.frame_effects = Some(vec!["showcase".to_owned()]);
        let mut foil_only = printing(Some("0.10"), (2021, 1, 1));
        foil_only.scryfall_data.finishes = vec!["foil".to_owned()];
        let mut full_art = printing(Some("0.10"), (2021, 1, 1));
        full_art.scryfall_data.full_art = true;
        let mut japanese = printing(Some("0.10"), (2021, 1, 1));
        japanese.scryfall_data.lang = "ja".to_owned();
        let mut digital = printing(Some("0.10"), (2021, 1, 1));
        digital.scryfall_data.digital = true;
        let mut art_series = printing(Some("0.10"), (2021, 1, 1));
        art_series.scryfall_data.layout = "art_series".to_owned();
        let printings = vec![
            promo,
            borderless,
            showcase,
            foil_only,
            full_art,
            japanese,
            digital,
            art_series,
            regular.clone(),
        ];
        assert_eq!(
            cheapest_regular(&printings).map(|c| c.scryfall_data.id),
            Some(regular.scryfall_data.id)
        );
    }

    #[test]
    fn no_priced_regular_printing_keeps_the_callers_pick() {
        let unpriced = printing(None, (2020, 1, 1));
        let mut promo = printing(Some("0.50"), (2020, 1, 1));
        promo.scryfall_data.promo = true;
        assert!(cheapest_regular(&[unpriced, promo]).is_none());
        assert!(cheapest_regular(&[]).is_none());
    }
}
