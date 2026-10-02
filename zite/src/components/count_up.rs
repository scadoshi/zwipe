//! A number that counts up from zero when it appears, and again whenever the
//! nav Z is clicked. The server render shows the final value, so the page
//! reads right before any script runs.

use crate::components::sleep_ms;
use dioxus::prelude::*;

/// How long a number takes to reach its value, and the tick between frames.
const DURATION_MS: f64 = 1000.0;
const TICK_MS: u32 = 16;

/// How many times the numbers have been asked to count again. Bumped by the
/// nav Z; every `CountUp` under it runs from zero once more.
#[derive(Clone, Copy)]
pub struct Replay(pub Signal<u32>);

/// A value that changes mid-count becomes the new target.
#[component]
pub fn CountUp(value: i64) -> Element {
    let replay = use_context::<Replay>().0;
    let mut target = use_signal(|| value);
    let mut shown = use_signal(|| value);
    use_effect(use_reactive!(|value| target.set(value)));
    use_effect(move || {
        let run = replay();
        spawn(async move {
            let frames = (DURATION_MS / f64::from(TICK_MS)).ceil();
            let mut frame = 0.0;
            while frame < frames {
                sleep_ms(TICK_MS).await;
                // A later replay owns the number now.
                if replay.peek().ne(&run) {
                    return;
                }
                frame += 1.0;
                let t = (frame / frames).min(1.0);
                let eased = 1.0 - (1.0 - t).powi(3);
                shown.set(scaled(*target.peek(), eased));
            }
            shown.set(*target.peek());
        });
    });
    rsx! { "{format_count(shown())}" }
}

/// 12345 -> "12,345"
pub fn format_count(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        out.push('-');
    }
    out.chars().rev().collect()
}

/// `value` at `fraction` of the way up, never past it. Counts are far below
/// 2^53, so the float conversion is exact.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn scaled(value: i64, fraction: f64) -> i64 {
    ((value as f64) * fraction.clamp(0.0, 1.0)).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_separators() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }

    #[test]
    fn the_number_rises_to_its_value_and_no_further() {
        assert_eq!(scaled(12_345, 0.0), 0);
        assert_eq!(scaled(12_345, 0.5), 6_173);
        assert_eq!(scaled(12_345, 1.0), 12_345);
        assert_eq!(scaled(12_345, 1.5), 12_345);
    }
}
