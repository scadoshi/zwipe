//! Hypergeometric draw-odds engine.
//!
//! Pure, dependency-free probability functions answering "what are the odds I
//! draw at least N cards of some category?" Drawing cards from a deck is
//! sampling *without replacement* from a finite population, which the
//! hypergeometric distribution models exactly.
//!
//! All functions share three population parameters:
//! - `n_deck` (`N`) — total cards in the deck,
//! - `k_category` (`K`) — cards belonging to the category of interest,
//! - `n_drawn` (`n`) — cards drawn (e.g. 7 for an opening hand).
//!
//! ```text
//! P(exactly k)   = C(K, k) * C(N-K, n-k) / C(N, n)
//! P(at least 1)  = 1 - C(N-K, n) / C(N, n)
//! P(at least t)  = 1 - sum_{k=0}^{t-1} P(exactly k)
//! ```
//!
//! Probabilities are computed via log-binomials / incremental ratios so
//! `C(99, 7)`-scale coefficients never overflow.
//!
//! This is a *baseline* model: it assumes a single random keep (no mulligans)
//! and no card selection (no scry/tutor/draw spells smoothing the deck). Frame
//! results honestly as "raw odds from a random draw."

/// Natural log of the binomial coefficient `C(n, k)`.
///
/// Returns `f64::NEG_INFINITY` (so `exp` is `0.0`) when `k > n`.
fn ln_choose(n: u32, k: u32) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    // Symmetry: C(n, k) == C(n, n-k); iterate the smaller side for fewer terms.
    let k = k.min(n - k);
    let mut acc = 0.0_f64;
    for i in 0..k {
        acc += f64::from(n - i).ln() - f64::from(i + 1).ln();
    }
    acc
}

/// `P(exactly k_drawn of the category in n_drawn cards)`.
///
/// Returns `0.0` for impossible draws (e.g. `k_drawn > k_category`,
/// `n_drawn > n_deck`, or too few non-category cards to fill the hand).
pub fn p_exactly(n_deck: u32, k_category: u32, n_drawn: u32, k_drawn: u32) -> f64 {
    let k_category = k_category.min(n_deck);
    if n_drawn > n_deck || k_drawn > k_category || k_drawn > n_drawn {
        return 0.0;
    }
    let others = n_deck - k_category;
    let from_others = n_drawn - k_drawn;
    if from_others > others {
        return 0.0;
    }
    let ln_p =
        ln_choose(k_category, k_drawn) + ln_choose(others, from_others) - ln_choose(n_deck, n_drawn);
    ln_p.exp()
}

/// `P(at least one of the category in n_drawn cards)` = `1 - C(N-K, n)/C(N, n)`.
///
/// Computed from the stable complement (probability of zero hits) via an
/// incremental ratio of falling factorials.
pub fn p_at_least_one(n_deck: u32, k_category: u32, n_drawn: u32) -> f64 {
    let k_category = k_category.min(n_deck);
    if k_category == 0 || n_drawn == 0 {
        return 0.0;
    }
    let others = n_deck - k_category;
    // Not enough non-category cards to fill the hand: a hit is guaranteed.
    if others < n_drawn {
        return 1.0;
    }
    // P(zero) = prod_{i=0}^{n-1} (N-K-i) / (N-i).
    let mut p_zero = 1.0_f64;
    for i in 0..n_drawn {
        p_zero *= f64::from(others - i) / f64::from(n_deck - i);
    }
    1.0 - p_zero
}

/// `P(at least t of the category in n_drawn cards)`.
///
/// `t == 0` is certain; `t == 1` delegates to the stable [`p_at_least_one`];
/// higher thresholds sum the exact-`k` tail `1 - sum_{k=0}^{t-1} P(exactly k)`.
pub fn p_at_least(n_deck: u32, k_category: u32, n_drawn: u32, t: u32) -> f64 {
    if t == 0 {
        return 1.0;
    }
    if t == 1 {
        return p_at_least_one(n_deck, k_category, n_drawn);
    }
    let mut cumulative = 0.0_f64;
    for k in 0..t {
        cumulative += p_exactly(n_deck, k_category, n_drawn, k);
    }
    (1.0 - cumulative).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert `actual` is within `1e-4` of `expected`.
    fn approx(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "expected ~{expected}, got {actual}"
        );
    }

    #[test]
    fn at_least_one_land_commander_opening_seven() {
        // 99-card commander deck, 37 lands, opening hand of 7.
        // 1 - C(62,7)/C(99,7) = 0.9670.
        approx(p_at_least_one(99, 37, 7), 0.9670);
    }

    #[test]
    fn at_least_one_land_commander_six_cards() {
        // Same deck drawing 6 (e.g. after one London mulligan to 6).
        approx(p_at_least_one(99, 37, 6), 0.9451);
    }

    #[test]
    fn at_least_one_land_sixty_card_opening_seven() {
        // Classic 60-card deck, 17 lands.
        approx(p_at_least_one(40, 17, 7), 0.9869);
    }

    #[test]
    fn at_least_two_lands_sixty_card() {
        // 60-card deck, 24 lands, at least two in the opening seven.
        approx(p_at_least(60, 24, 7, 2), 0.8573);
    }

    #[test]
    fn exactly_zero_is_complement_of_at_least_one() {
        let zero = p_exactly(99, 37, 7, 0);
        approx(zero, 0.0330);
        approx(zero + p_at_least_one(99, 37, 7), 1.0);
    }

    #[test]
    fn pmf_sums_to_one() {
        // The full distribution over possible hit counts must sum to 1.
        let total: f64 = (0..=7).map(|k| p_exactly(40, 17, 7, k)).sum();
        approx(total, 1.0);
    }

    #[test]
    fn at_least_is_monotonically_non_increasing_in_threshold() {
        let mut prev = 1.0;
        for t in 0..=7 {
            let p = p_at_least(40, 17, 7, t);
            assert!(p <= prev + 1e-9, "t={t}: {p} should be <= {prev}");
            prev = p;
        }
    }

    #[test]
    fn empty_category_never_hits() {
        assert_eq!(p_at_least_one(99, 0, 7), 0.0);
        assert_eq!(p_at_least(99, 0, 7, 1), 0.0);
        assert_eq!(p_exactly(99, 0, 7, 1), 0.0);
    }

    #[test]
    fn drawing_nothing_never_hits() {
        assert_eq!(p_at_least_one(99, 37, 0), 0.0);
        assert_eq!(p_exactly(99, 37, 0, 0), 1.0);
    }

    #[test]
    fn whole_category_or_more_guarantees_a_hit() {
        // Every card is in the category → any nonempty draw hits.
        approx(p_at_least_one(40, 40, 7), 1.0);
        // Fewer non-category cards than the hand → a hit is guaranteed.
        approx(p_at_least_one(10, 5, 7), 1.0);
    }

    #[test]
    fn threshold_zero_is_certain() {
        assert_eq!(p_at_least(99, 37, 7, 0), 1.0);
    }

    #[test]
    fn impossible_exact_counts_are_zero() {
        // Can't draw more of the category than exist.
        assert_eq!(p_exactly(40, 3, 7, 5), 0.0);
        // Can't draw more cards than the deck holds.
        assert_eq!(p_exactly(5, 2, 7, 1), 0.0);
        // Can't draw more category cards than cards drawn.
        assert_eq!(p_exactly(40, 17, 3, 4), 0.0);
    }

    #[test]
    fn oversized_category_is_clamped_to_deck() {
        // K > N is nonsensical input; treat as "all cards are the category."
        approx(p_at_least_one(40, 99, 7), 1.0);
    }

    #[test]
    fn single_card_category_matches_direct_ratio() {
        // 1 specific card in a 99-card deck, opening 7: exactly 7/99.
        approx(p_at_least_one(99, 1, 7), 7.0 / 99.0);
    }
}
