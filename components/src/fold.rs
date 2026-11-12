//! The fold under a long chart: the rows past the first few ease open from a
//! See more chip and shut again from See less. Shared by the deck charts so
//! the app and the shared deck page fold the same way.

use dioxus::prelude::*;

/// How many rows a chart shows before the rest fold away.
pub const FOLD_AT: usize = 5;

/// The See more chip's text while the fold is shut.
pub const SEE_MORE: &str = "See more";

/// The See more chip's text while the fold is open.
pub const SEE_LESS: &str = "See less";

/// `items` split at `fold_at`: the rows always shown, then the rows behind
/// the fold, `None` when nothing needs folding. With `fold_at` rows or fewer
/// everything is shown and there is no fold.
pub fn fold_split<T>(items: &[T], fold_at: usize) -> (&[T], Option<&[T]>) {
    if items.len() <= fold_at {
        return (items, None);
    }
    let (shown, rest) = items.split_at(fold_at);
    (shown, Some(rest))
}

/// The folding wrapper: `.chart-fold`, eased between shut and open with the
/// grid-rows 0fr to 1fr transition in `magic.css`. Mount it shut with its
/// content inside so opening is a class change the transition can ease.
/// Shut, it is inert.
///
/// Inside a grid of chart rows, `subgrid` makes the fold and its inner span
/// and share the grid's columns, so the rows behind it line up with the rows
/// before it and size the columns while shut.
#[component]
pub fn Fold(open: bool, #[props(default)] subgrid: bool, children: Element) -> Element {
    let class = match (open, subgrid) {
        (true, true) => "chart-fold open subgrid",
        (true, false) => "chart-fold open",
        (false, true) => "chart-fold subgrid",
        (false, false) => "chart-fold",
    };
    // A shut fold is inert, as every fold is: Tab and screen readers pass
    // over its rows, and its charts' entrance waits for it to open.
    rsx! {
        div { class, inert: (!open).then_some(true),
            div { class: "chart-fold-inner", {children} }
        }
    }
}

/// The chip under a folded chart: a centered `btn-action` reading See more
/// while the fold is shut and See less while it is open.
#[component]
pub fn SeeMore(open: bool, on_toggle: Callback<()>) -> Element {
    let text = if open { SEE_LESS } else { SEE_MORE };
    rsx! {
        div { class: "chart-see-more",
            button {
                class: "btn-action",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_toggle.call(());
                },
                "{text}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_or_fewer_rows_do_not_fold() {
        let five = [1, 2, 3, 4, 5];
        assert_eq!(fold_split(&five, FOLD_AT), (&five[..], None));
        let none: [i32; 0] = [];
        assert_eq!(fold_split(&none, FOLD_AT), (&none[..], None));
    }

    #[test]
    fn a_sixth_row_folds_alone() {
        let six = [1, 2, 3, 4, 5, 6];
        assert_eq!(
            fold_split(&six, FOLD_AT),
            (&six[..5], Some(&six[5..])),
            "the first five show, the sixth is behind the fold"
        );
    }
}
