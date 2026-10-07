//! The arithmetic behind the hand-drawn SVG charts: smooth lines, where a
//! hover chip hangs, and which days of a heatmap stand out. Pure functions,
//! no rendering.

use std::fmt::Write as _;

/// A smooth path through `points` that passes through every point and never
/// overshoots between two of them: Fritsch-Carlson monotone cubic
/// interpolation as cubic Bezier segments. A flat run into a spike stays flat
/// rather than dipping below the data first. Empty for no points.
pub fn curve(points: &[(f64, f64)]) -> String {
    let Some(first) = points.first() else {
        return String::new();
    };
    let mut d = format!("M {:.1} {:.1}", first.0, first.1);
    if points.len() < 2 {
        return d;
    }
    // Secant slopes between neighbors, then a tangent per point that keeps
    // each segment monotone.
    let deltas: Vec<f64> = points
        .windows(2)
        .map(|pair| match pair {
            [a, b] => {
                let dx = b.0 - a.0;
                if dx.abs() < f64::EPSILON {
                    0.0
                } else {
                    (b.1 - a.1) / dx
                }
            }
            _ => 0.0,
        })
        .collect();
    let mut tangents = Vec::with_capacity(points.len());
    tangents.push(deltas.first().copied().unwrap_or(0.0));
    for pair in deltas.windows(2) {
        if let [a, b] = pair {
            tangents.push(if a * b <= 0.0 {
                0.0
            } else {
                f64::midpoint(*a, *b)
            });
        }
    }
    tangents.push(deltas.last().copied().unwrap_or(0.0));
    for (i, &delta) in deltas.iter().enumerate() {
        if delta.abs() < f64::EPSILON {
            set(&mut tangents, i, 0.0);
            set(&mut tangents, i + 1, 0.0);
            continue;
        }
        let (Some(&here), Some(&next)) = (tangents.get(i), tangents.get(i + 1)) else {
            continue;
        };
        let alpha = here / delta;
        let beta = next / delta;
        let size = alpha.hypot(beta);
        if size > 3.0 {
            let scale = 3.0 / size;
            set(&mut tangents, i, scale * alpha * delta);
            set(&mut tangents, i + 1, scale * beta * delta);
        }
    }
    for (pair, slopes) in points.windows(2).zip(tangents.windows(2)) {
        let ([p1, p2], [t1, t2]) = (pair, slopes) else {
            continue;
        };
        let dx = (p2.0 - p1.0) / 3.0;
        let c1 = (p1.0 + dx, p1.1 + t1 * dx);
        let c2 = (p2.0 - dx, p2.1 - t2 * dx);
        let _ = write!(
            d,
            " C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}",
            c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
        );
    }
    d
}

/// Writes `value` at `i` when `i` is in range.
fn set(values: &mut [f64], i: usize, value: f64) {
    if let Some(slot) = values.get_mut(i) {
        *slot = value;
    }
}

/// [`curve`] closed down to `baseline`, for the area under a line.
pub fn area(points: &[(f64, f64)], baseline: f64) -> String {
    match (points.first(), points.last()) {
        (Some(first), Some(last)) => format!(
            "{} L {:.1} {baseline:.1} L {:.1} {baseline:.1} Z",
            curve(points),
            last.0,
            first.0
        ),
        _ => String::new(),
    }
}

/// Which way a hover chip hangs off its point, given the point's position
/// as a percentage of the chart's width: centered in the middle, and from
/// its edge near either side (`tip-start`, `tip-end`) so it never leaves the
/// chart.
pub fn tip_anchor(left: f64) -> &'static str {
    if left < 15.0 {
        "tip-start"
    } else if left > 85.0 {
        "tip-end"
    } else {
        ""
    }
}

/// The outliers among `counts`, as indices into it: the days at or past
/// `ratio` times the median of the nonzero days, the largest `max` of them,
/// biggest first (ties in their original order). A steady busy stretch never
/// qualifies; a spike does. Zero days never count, nor drag the median down.
pub fn peak_indices(counts: &[u32], ratio: u32, max: usize) -> Vec<usize> {
    let mut busy: Vec<u32> = counts.iter().copied().filter(|&n| n > 0).collect();
    if busy.is_empty() {
        return Vec::new();
    }
    busy.sort_unstable();
    let median = busy.get(busy.len() / 2).copied().unwrap_or(0);
    let floor = median.saturating_mul(ratio);
    let mut peaks: Vec<(usize, u32)> = counts
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, n)| n > 0 && n >= floor)
        .collect();
    peaks.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    peaks.truncate(max);
    peaks.into_iter().map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every y in a path's control points and anchors.
    fn ys(path: &str) -> Vec<f64> {
        path.split([',', ' '])
            .filter_map(|token| token.parse::<f64>().ok())
            .skip(1)
            .step_by(2)
            .collect()
    }

    #[test]
    fn the_curve_starts_at_the_first_point_and_the_area_closes_on_the_baseline() {
        let points = [(0.0, 10.0), (10.0, 0.0), (20.0, 10.0)];
        assert!(curve(&points).starts_with("M 0.0 10.0 C"));
        assert_eq!(curve(&points).matches(" C ").count(), 2);
        let area = area(&points, 12.0);
        assert!(area.ends_with("L 20.0 12.0 L 0.0 12.0 Z"), "{area}");
        assert_eq!(curve(&[]), "");
        assert_eq!(curve(&[(3.0, 4.0)]), "M 3.0 4.0");
    }

    #[test]
    fn a_flat_run_into_a_spike_never_dips_below_the_flat() {
        // In SVG a larger y is lower on the page: the baseline is 100 and the
        // spike rises to 10. No control point may go past 100.
        let points = [
            (0.0, 100.0),
            (10.0, 100.0),
            (20.0, 100.0),
            (30.0, 10.0),
            (40.0, 100.0),
        ];
        let lowest = ys(&curve(&points)).into_iter().fold(0.0_f64, f64::max);
        assert!(lowest <= 100.0, "a control point overshot to {lowest}");
    }

    #[test]
    fn the_chip_hangs_from_its_edge_near_either_side() {
        assert_eq!(tip_anchor(5.0), "tip-start");
        assert_eq!(tip_anchor(50.0), "");
        assert_eq!(tip_anchor(95.0), "tip-end");
    }

    #[test]
    fn peaks_are_the_spikes_past_four_times_the_median_busy_day() {
        // Busy days 2, 3, 4, 20, 40: the median is 4, the floor 16, so only
        // the 20 and the 40 qualify, biggest first, and the zero day never
        // drags the median down.
        assert_eq!(peak_indices(&[0, 2, 3, 4, 20, 40], 4, 12), [5, 4]);
        assert_eq!(peak_indices(&[0], 4, 12), [0usize; 0]);
        assert_eq!(peak_indices(&[], 4, 12), [0usize; 0]);
    }

    #[test]
    fn a_steady_stretch_has_no_peaks_and_a_spike_does() {
        let mut counts = vec![10; 20];
        counts.push(45);
        assert_eq!(peak_indices(&counts, 4, 12), [20]);
    }

    #[test]
    fn only_the_biggest_peaks_are_kept() {
        assert_eq!(peak_indices(&[1, 1, 1, 1, 1, 9, 8, 7], 4, 2), [5, 6]);
    }
}
