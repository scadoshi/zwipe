//! The quantity stepper's `+` is disabled once a card holds 99 copies, or once the deck its page
//! provides a [`components::CopyRoom`] for is at its card limit; `-` always works, so a card or
//! deck already past a limit can come down.

#![expect(
    clippy::unwrap_used,
    reason = "a test: a helper outside #[test] fns unwraps, and a panic is the failure report"
)]

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The stepper's two buttons, `-` then `+`, as rendered.
fn steppers(html: &str) -> (String, String) {
    let buttons: Vec<&str> = html.split("<button").skip(1).collect();
    let minus = buttons.iter().find(|b| b.contains(">-</button>")).unwrap();
    let plus = buttons.iter().find(|b| b.contains(">+</button>")).unwrap();
    ((*minus).to_owned(), (*plus).to_owned())
}

fn stepper_at(qty: i32) -> Element {
    rsx! { components::CardEditButtons { qty, on_qty_change: move |_: i32| {} } }
}

#[test]
fn plus_works_below_99_copies() {
    let (minus, plus) = steppers(&render(|| stepper_at(98)));
    assert!(!plus.contains("disabled"), "{plus}");
    assert!(!minus.contains("disabled"), "{minus}");
}

#[test]
fn plus_is_disabled_at_99_copies_and_past_them() {
    for html in [render(|| stepper_at(99)), render(|| stepper_at(120))] {
        let (minus, plus) = steppers(&html);
        assert!(plus.contains("disabled"), "{plus}");
        assert!(!minus.contains("disabled"), "{minus}");
    }
}

#[test]
fn plus_is_disabled_once_the_deck_is_at_its_limit() {
    fn full() -> Element {
        use_context_provider(|| {
            components::CopyRoom::new(
                ReadSignal::from(Signal::new(100)),
                ReadSignal::from(Signal::new(100)),
            )
        });
        stepper_at(3)
    }
    fn room() -> Element {
        use_context_provider(|| {
            components::CopyRoom::new(
                ReadSignal::from(Signal::new(99)),
                ReadSignal::from(Signal::new(100)),
            )
        });
        stepper_at(3)
    }
    let (minus, plus) = steppers(&render(full));
    assert!(plus.contains("disabled"), "{plus}");
    assert!(!minus.contains("disabled"), "{minus}");
    let (_, plus) = steppers(&render(room));
    assert!(!plus.contains("disabled"), "{plus}");
}
