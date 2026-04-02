//! Oracle text filter component.
//!
//! Provides oracle text contains (free text), oracle words contains (chip multi-select),
//! and keywords contains (chip multi-select), each with any/all matching toggles.

/// Keywords chip multi-select.
pub(crate) mod keywords;
/// Oracle words chip multi-select.
pub(crate) mod oracle_words;
/// Free-text oracle text search.
pub(crate) mod text_contains;

use dioxus::prelude::*;
use keywords::Keywords;
use oracle_words::OracleWords;
use text_contains::TextContains;

/// Filter component for oracle text search, oracle words, and keywords.
#[component]
pub fn OracleText() -> Element {
    rsx! {
        div { class: "flex-col gap-half",
            TextContains {}
            OracleWords {}
            Keywords {}
        }
    }
}
