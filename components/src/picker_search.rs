//! A picker sheet's search text, which never outlives the sheet.

use dioxus::prelude::*;

/// A picker's search text, emptied whenever `open` turns false (Save,
/// Discard or a quiet close), so the picker reopens on its whole list. For a
/// picker that stays mounted while shut.
pub fn use_picker_search(open: ReadSignal<bool>) -> Signal<String> {
    let mut query = use_signal(String::new);
    use_effect(move || {
        if !open() && !query.peek().is_empty() {
            query.set(String::new());
        }
    });
    query
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    thread_local! {
        static SIGNALS: RefCell<Option<(Signal<bool>, Signal<String>)>> = const { RefCell::new(None) };
    }

    #[component]
    fn Picker() -> Element {
        let open = use_signal(|| true);
        let query = use_picker_search(open.into());
        SIGNALS.with(|s| *s.borrow_mut() = Some((open, query)));
        rsx! {}
    }

    #[test]
    fn closing_a_picker_empties_its_search() {
        let mut dom = VirtualDom::new(Picker);
        dom.rebuild_in_place();
        let (mut open, mut query) = SIGNALS.with(|s| s.borrow().unwrap());
        dom.in_runtime(|| query.set("pau".to_owned()));
        dom.render_immediate_to_vec();
        assert_eq!(
            dom.in_runtime(|| query.peek().clone()),
            "pau",
            "typing while open stays"
        );
        dom.in_runtime(|| open.set(false));
        dom.render_immediate_to_vec();
        assert_eq!(
            dom.in_runtime(|| query.peek().clone()),
            "",
            "closed, the search is empty"
        );
    }
}
