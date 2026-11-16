//! A flip re-renders only the flipped card's surfaces: each surface reads its own card's face out
//! of the host's faces map (`use_shared_face`), not the whole map.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::cell::Cell;
use std::collections::HashMap;

use components::use_shared_face;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;

/// How many rows the list holds.
const ROWS: u128 = 30;

thread_local! {
    static FACES: Cell<Option<Signal<HashMap<Uuid, usize>>>> = const { Cell::new(None) };
    static RENDERS: Cell<usize> = const { Cell::new(0) };
}

/// A row that shows card `id` on the face the host keeps for it.
#[component]
fn Row(id: Uuid, faces: Signal<HashMap<Uuid, usize>>) -> Element {
    RENDERS.with(|r| r.set(r.get().saturating_add(1)));
    let face = use_shared_face(Some(faces), id).unwrap();
    rsx! {
        div { "{id}: {face}" }
    }
}

fn app() -> Element {
    let faces = use_signal(HashMap::new);
    FACES.with(|f| f.set(Some(faces)));
    rsx! {
        for n in 0..ROWS {
            Row { key: "{n}", id: Uuid::from_u128(n), faces }
        }
    }
}

fn settle(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
}

#[test]
fn a_flip_re_renders_only_the_flipped_row() {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    settle(&mut dom);
    RENDERS.with(|r| r.set(0));
    let mut faces = FACES.with(Cell::get).unwrap();
    dom.in_runtime(|| {
        faces.write().insert(Uuid::from_u128(4), 1);
    });
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains(&format!("{}: 1", Uuid::from_u128(4))),
        "the flipped row shows its other face: {html}"
    );
    let rendered = RENDERS.with(Cell::get);
    assert!(
        rendered <= 2,
        "the flipped row renders (once for the map, once for its face), not all {ROWS}: {rendered}"
    );
}
