//! Hypergeometric draw odds: the probability of drawing cards of a category
//! from a deck **without replacement**.
//!
//! Pure math, no dependencies. Given a `deck_size`-card deck holding
//! `successes` cards of some category (lands, ramp, 3-drops, …), these compute
//! the odds of seeing them when drawing `draws` cards (opening 7, by turn N, …).
//!
//! This is a **baseline** model: it's the raw random-draw probability. It does
//! **not** account for mulligans or card selection (tutors / scry / card draw),
//! so present it as "odds from a random draw, pre-mulligan".
//!
//! Numerically stable via log-factorials, so `C(100, 7)`-scale intermediates
//! never overflow.

/// Natural log of `n!`, as the sum of `ln(i)`. Accurate and overflow-free for
/// the deck-sized inputs here (at most a few hundred cards). `0! = 1! = 1`.
fn ln_factorial(n: u32) -> f64 {
    (2..=n).map(|i| f64::from(i).ln()).sum()
}

/// Natural log of the binomial coefficient `C(n, k)`. Returns
/// [`f64::NEG_INFINITY`] (i.e. `C = 0`) when `k > n`.
fn ln_choose(n: u32, k: u32) -> f64 {
    let Some(rest) = n.checked_sub(k) else {
        return f64::NEG_INFINITY;
    };
    ln_factorial(n) - ln_factorial(k) - ln_factorial(rest)
}

/// `P(exactly k of the category)` when drawing `draws` cards from a `deck_size`
/// deck containing `successes` of the category. Returns `0.0` for an impossible
/// `k` (more than exist, or too many to leave room for the rest of the hand).
/// `draws` and `successes` are both capped at `deck_size`: you can't draw more
/// cards than the deck holds, and a category can't hold more cards than the deck.
pub fn p_exactly(deck_size: u32, successes: u32, draws: u32, k: u32) -> f64 {
    // Cap first. A 0-card draw (empty deck, or draws == 0) yields exactly 0
    // successes with certainty; the general computation below produces that
    // (C(·,0) = 1); no special-case.
    let draws = draws.min(deck_size);
    let successes = successes.min(deck_size);
    // `draws - k` is `None` when k > draws, an impossible k.
    let (Some(others), Some(other_draws)) =
        (deck_size.checked_sub(successes), draws.checked_sub(k))
    else {
        return 0.0;
    };
    // k of the successes, and (draws - k) of the non-successes, must both fit.
    if k > successes || other_draws > others {
        return 0.0;
    }
    let ln_p =
        ln_choose(successes, k) + ln_choose(others, other_draws) - ln_choose(deck_size, draws);
    ln_p.exp().clamp(0.0, 1.0)
}

/// `P(at least `threshold` of the category)` in `draws` cards.
/// `threshold == 0` is certain (`1.0`).
pub fn p_at_least(deck_size: u32, successes: u32, draws: u32, threshold: u32) -> f64 {
    if threshold == 0 {
        return 1.0;
    }
    // 1 − P(fewer than threshold) = 1 − Σ_{k=0}^{threshold-1} P(exactly k).
    let below: f64 = (0..threshold)
        .map(|k| p_exactly(deck_size, successes, draws, k))
        .sum();
    (1.0 - below).clamp(0.0, 1.0)
}

/// Convenience for the headline question: `P(at least one of the category)`.
pub fn p_at_least_one(deck_size: u32, successes: u32, draws: u32) -> f64 {
    p_at_least(deck_size, successes, draws, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    #[test]
    fn single_draw_is_k_over_n() {
        // Drawing one card, P(>=1) is just the fraction of the deck.
        assert!((p_at_least_one(99, 37, 1) - 37.0 / 99.0).abs() < EPS);
    }

    #[test]
    fn known_land_odds_37_of_99_draw_7() {
        // Classic Commander check: 37 lands in 99, opening 7 → ~96.7% for >=1 land.
        let p = p_at_least_one(99, 37, 7);
        assert!((p - 0.966_960).abs() < 1e-4, "got {p}");
    }

    #[test]
    fn all_or_nothing() {
        assert!((p_at_least_one(40, 40, 7) - 1.0).abs() < EPS); // every card qualifies
        assert_eq!(p_at_least_one(40, 0, 7), 0.0); // none qualify
    }

    #[test]
    fn draws_capped_at_deck_size() {
        // "Draw 10" from a 5-card deck = draw all 5; the 2 successes are guaranteed.
        assert!((p_at_least_one(5, 2, 10) - 1.0).abs() < EPS);
    }

    #[test]
    fn pmf_sums_to_one() {
        let (n, k, draws) = (60, 24, 7);
        let total: f64 = (0..=draws).map(|i| p_exactly(n, k, draws, i)).sum();
        assert!((total - 1.0).abs() < 1e-9, "pmf summed to {total}");
    }

    #[test]
    fn at_least_two_never_exceeds_at_least_one() {
        let p1 = p_at_least(99, 37, 7, 1);
        let p2 = p_at_least(99, 37, 7, 2);
        assert!(p2 <= p1 && p2 >= 0.0);
    }

    #[test]
    fn threshold_zero_is_certain() {
        assert_eq!(p_at_least(99, 37, 7, 0), 1.0);
    }

    #[test]
    fn empty_deck_has_no_successes() {
        assert_eq!(p_at_least_one(0, 0, 7), 0.0); // can't draw a card that isn't there
        assert_eq!(p_exactly(0, 0, 7, 1), 0.0); // no success to draw
        // Drawing nothing yields exactly zero successes with certainty.
        assert!((p_exactly(0, 0, 7, 0) - 1.0).abs() < EPS);
    }

    #[test]
    fn valid_inputs_keep_their_exact_values() {
        let cases = [
            (
                (99, 37, 7, 1),
                0.966_964_794_119_874_8,
                0.152_787_827_195_582_6,
            ),
            (
                (60, 24, 7, 1),
                0.978_385_472_740_881_5,
                0.121_041_352_651_063_15,
            ),
            (
                (60, 24, 10, 2),
                0.966_660_149_919_705_6,
                0.110_776_066_879_950_52,
            ),
            (
                (40, 17, 9, 3),
                0.844_594_594_594_594_5,
                0.251_039_501_039_502_46,
            ),
            (
                (100, 10, 7, 1),
                0.533_259_585_682_814_5,
                0.388_950_345_264_343_24,
            ),
            ((5, 2, 10, 1), 1.0, 0.0),
        ];
        for ((n, s, d, k), at_least, exactly) in cases {
            assert_eq!(
                p_at_least(n, s, d, k),
                at_least,
                "p_at_least({n}, {s}, {d}, {k})"
            );
            assert_eq!(
                p_exactly(n, s, d, k),
                exactly,
                "p_exactly({n}, {s}, {d}, {k})"
            );
        }
    }

    #[test]
    fn more_successes_than_cards_is_a_full_deck_of_them() {
        assert_eq!(p_at_least_one(60, 70, 7), 1.0);
        assert_eq!(p_at_least_one(5, 9, 10), 1.0);
        assert_eq!(p_exactly(60, 70, 7, 7), 1.0);
        assert_eq!(p_exactly(60, 70, 7, 3), 0.0);
        assert_eq!(p_at_least_one(0, 3, 7), 0.0); // an empty deck still draws nothing
    }
}
