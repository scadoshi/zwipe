//! Which printing Quick add lands: a card's cheapest regular printing, not its newest.
//!
//! Regular means what import lands for a line that names no printing (English, paper, regular
//! size, not a promo, sold in nonfoil, never an Art Series, token or memorabilia printing) and no
//! special treatment on top: no borderless, full-art, textless, showcase, extended-art, etched or
//! inverted frame. Cheapest is by the USD price; a tie goes to the newest. A USD price under a
//! tenth of the same printing's EUR price reads as a bad figure, and that printing is skipped; a
//! printing with only one of the two prices is judged by its USD price alone.

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

/// A USD price below the EUR price divided by this reads as a bad figure.
const USD_UNDER_EUR_DIVISOR: f64 = 10.0;

/// The USD price of `printing`, when it has one that reads as a number and agrees with its EUR
/// price (see the module doc).
fn usd(printing: &ScryfallData) -> Option<f64> {
    let usd = printing.prices.usd.as_deref()?.parse::<f64>().ok()?;
    let eur = printing
        .prices
        .eur
        .as_deref()
        .and_then(|eur| eur.parse::<f64>().ok());
    match eur {
        Some(eur) if usd < eur / USD_UNDER_EUR_DIVISOR => None,
        Some(_) | None => Some(usd),
    }
}

/// The cheapest regular printing among `printings`, the newest of any tied; `None` when no
/// regular printing has a USD price it can trust, and the caller keeps the printing it has.
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
    fn a_usd_price_under_a_tenth_of_eur_is_skipped() {
        // Summer Magic Lightning Bolt: $0.10 next to €5017.02.
        let mut summer_magic = printing(Some("0.10"), (1994, 7, 1));
        summer_magic.scryfall_data.prices.eur = Some("5017.02".to_owned());
        let mut foundations = printing(Some("0.82"), (2024, 11, 15));
        foundations.scryfall_data.prices.eur = Some("1.67".to_owned());
        let printings = [summer_magic, foundations.clone()];
        assert_eq!(
            cheapest_regular(&printings).map(|c| c.scryfall_data.id),
            Some(foundations.scryfall_data.id),
            "the bad USD figure loses to the next cheapest"
        );
    }

    #[test]
    fn a_printing_with_one_currency_or_close_prices_is_judged_by_usd() {
        let usd_only = printing(Some("0.10"), (2020, 1, 1));
        let mut eur_close = printing(Some("0.20"), (2021, 1, 1));
        eur_close.scryfall_data.prices.eur = Some("2.00".to_owned());
        let printings = [usd_only.clone(), eur_close.clone()];
        assert_eq!(
            cheapest_regular(&printings).map(|c| c.scryfall_data.id),
            Some(usd_only.scryfall_data.id),
            "no EUR price: the USD price stands"
        );
        assert_eq!(
            cheapest_regular(&[eur_close.clone()]).map(|c| c.scryfall_data.id),
            Some(eur_close.scryfall_data.id),
            "USD at exactly a tenth of EUR still counts"
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
