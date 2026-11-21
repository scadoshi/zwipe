//! A Show more list's rows and ghost rows draw in one loop whose body's first node is a keyed
//! `Fragment`, as the Commanders screen's lists on the app and the web draw them: each row keeps
//! its own component, with its own state, as rows around it leave or a later Show more moves the
//! ghosts.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::commander::more_page::{MoreItem, MorePage};

/// A row that remembers the id it mounted with, so a row whose component took another row's props
/// shows two different ids.
#[component]
fn Row(id: u128) -> Element {
    let mounted = use_signal(|| id);
    rsx! { p { "{mounted}={id}" } }
}

/// The rows the list holds, in a signal the test edits.
static ROWS: GlobalSignal<Vec<u128>> = Signal::global(|| (1..=25).collect());
/// The list's Show more state.
static PAGE: GlobalSignal<MorePage> = Signal::global(MorePage::new);

fn oracle(row: &u128) -> Uuid {
    Uuid::from_u128(*row)
}

fn list() -> Element {
    let rows = ROWS.read().clone();
    let layout = PAGE.read().layout(&rows, |row| Some(oracle(row)));
    let items = layout
        .into_iter()
        .map(|item| (item.key(ToString::to_string), item));
    rsx! {
        for (key, item) in items {
            Fragment { key: "{key}",
                {match item {
                    MoreItem::Row(id) => rsx! { Row { id } },
                    MoreItem::Ghosts => rsx! { hr {} },
                }}
            }
        }
    }
}

/// Every row's text, `mounted=current`.
fn rows_drawn(dom: &VirtualDom) -> Vec<String> {
    let html = dioxus_ssr::render(dom);
    html.split("<p>")
        .skip(1)
        .filter_map(|p| p.split("</p>").next())
        .map(str::to_owned)
        .collect()
}

fn settle(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut NoOpMutations);
}

#[test]
fn a_removed_row_leaves_every_other_row_its_own_state() {
    let mut dom = VirtualDom::new(list);
    dom.rebuild_in_place();
    dom.in_runtime(|| ROWS.write().retain(|row| *row != 3));
    settle(&mut dom);
    let drawn = rows_drawn(&dom);
    assert!(
        drawn.iter().all(|r| !r.ends_with("=3")),
        "the remove drew: {drawn:?}"
    );
    assert_eq!(drawn.len(), 10, "{drawn:?}");
    for row in drawn {
        let (mounted, current) = row.split_once('=').unwrap();
        assert_eq!(
            mounted, current,
            "a row took its neighbor's component: {row}"
        );
    }
}

#[test]
fn a_later_show_more_leaves_the_rows_before_it_their_own_state() {
    let mut dom = VirtualDom::new(list);
    dom.rebuild_in_place();
    for _ in 0..2 {
        dom.in_runtime(|| {
            let rows = ROWS.read().clone();
            PAGE.write().show_more(&rows, |row| Some(oracle(row)));
            PAGE.write()
                .animation_ended(zwipe_ui::commander::more_ghosts::ENTER_ANIMATION);
        });
        settle(&mut dom);
    }
    let drawn = rows_drawn(&dom);
    assert_eq!(drawn.len(), 25, "{drawn:?}");
    for row in drawn {
        let (mounted, current) = row.split_once('=').unwrap();
        assert_eq!(
            mounted, current,
            "a row took another row's component: {row}"
        );
    }
}
