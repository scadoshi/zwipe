//! Price range filter component (min/max in a selected currency).

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;

/// Price filter sub-component: USD/EUR/TIX chips + min/max bounds.
#[component]
pub(crate) fn PriceFilter() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    let toast = use_toast();

    // Input buffers, seeded from the builder so the screen survives re-mounts.
    let mut price_min_string = use_signal(|| {
        filter_builder()
            .price_min()
            .map(|v| v.to_string())
            .unwrap_or_default()
    });
    let mut price_max_string = use_signal(|| {
        filter_builder()
            .price_max()
            .map(|v| v.to_string())
            .unwrap_or_default()
    });

    // Parse and write the min bound on blur (empty clears it).
    let mut try_parse_price_min = move || {
        if price_min_string().is_empty() {
            filter_builder.write().unset_price_min();
            return;
        }
        if let Ok(n) = price_min_string().parse::<f64>() {
            filter_builder.write().set_price_min(n);
            price_min_string.set(n.to_string());
        } else {
            toast.error(
                "Invalid price".to_string(),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        }
    };

    // Parse and write the max bound on blur (empty clears it).
    let mut try_parse_price_max = move || {
        if price_max_string().is_empty() {
            filter_builder.write().unset_price_max();
            return;
        }
        if let Ok(n) = price_max_string().parse::<f64>() {
            filter_builder.write().set_price_max(n);
            price_max_string.set(n.to_string());
        } else {
            toast.error(
                "Invalid price".to_string(),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        }
    };

    let selected_currency = filter_builder().price_currency().unwrap_or_default();
    let price_is_active =
        filter_builder().price_min().is_some() || filter_builder().price_max().is_some();

    rsx! {
        // Single wrapper so the accordion's grid-row collapse animates the whole
        // section as one unit (multiple top-level children only collapse the
        // first grid row, leaving the rest to squeeze).
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", "Price" }
                if price_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let mut fb = filter_builder.write();
                            fb.unset_price_min();
                            fb.unset_price_max();
                            price_min_string.set(String::new());
                            price_max_string.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "chip-row", style: "margin-bottom: 0.5rem; justify-content: center;",
                for currency in PriceCurrency::all().iter().copied() {
                    div {
                        class: if selected_currency == currency { "chip selected" } else { "chip" },
                        onclick: move |_| {
                            filter_builder.write().set_price_currency(currency);
                        },
                        "{currency.label()}"
                    }
                }
            }

            div { class: "flex-row gap-1 flex-center mb-1",
                input { class: "input input-compact input-narrow",
                    id: "price-min",
                    placeholder: "Min",
                    value: price_min_string(),
                    r#type: "text",
                    inputmode: "decimal",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| {
                        price_min_string.set(event.value())
                    },
                    onblur: move |_| {
                        try_parse_price_min();
                    }
                }
                span { class: "text-muted", "to" }
                input { class: "input input-compact input-narrow",
                    id: "price-max",
                    placeholder: "Max",
                    value: price_max_string(),
                    r#type: "text",
                    inputmode: "decimal",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| {
                        price_max_string.set(event.value())
                    },
                    onblur: move |_| {
                        try_parse_price_max();
                    }
                }
            }
        }
    }
}
