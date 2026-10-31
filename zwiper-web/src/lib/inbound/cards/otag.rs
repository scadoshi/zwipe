//! The oracle tag reveals: a tapped tag's description from the catalog, and
//! the `Examples` button that opens the example cards over the screen.
//!
//! A screen that shows tags calls [`use_otag_reveals`] and mounts
//! [`OtagExamplesHost`]; every card row, chart and table under it reads the
//! two contexts this provides.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;

use crate::inbound::catalog::CatalogCache;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// The description lookup the tag reveals call, by slug.
#[derive(Clone, Copy)]
#[expect(
    missing_debug_implementations,
    reason = "a context holding a callback, which has no Debug"
)]
pub struct OtagDescribe(pub Callback<String, Option<String>>);

/// Opens the example cards for a slug.
#[derive(Clone, Copy)]
#[expect(
    missing_debug_implementations,
    reason = "a context holding a callback, which has no Debug"
)]
pub struct OtagExamplesOpen(pub Callback<String>);

/// Which tag's examples are open over the screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OtagExamples {
    /// Whether the overlay shows.
    pub open: Signal<bool>,
    /// The tag it shows.
    pub slug: Signal<String>,
}

/// Looks up tag descriptions in the root catalog, fetching the oracle tag
/// list on mount when it needs it.
pub fn use_describe_tag() -> Callback<String, Option<String>> {
    let cache: CatalogCache = use_context();
    let client: Signal<ZwipeClient> = use_context();
    use_hook(move || cache.ensure_oracle_tags(client));
    use_callback(move |slug: String| cache.describe(&slug))
}

/// Provides [`OtagDescribe`] and [`OtagExamplesOpen`] to the screen and
/// returns the examples state for its [`OtagExamplesHost`].
pub fn use_otag_reveals() -> OtagExamples {
    let describe = use_describe_tag();
    use_context_provider(|| OtagDescribe(describe));
    let examples = OtagExamples {
        open: use_signal(|| false),
        slug: use_signal(String::new),
    };
    let OtagExamples { mut open, mut slug } = examples;
    let opener = use_callback(move |tag: String| {
        slug.set(tag);
        open.set(true);
    });
    use_context_provider(|| OtagExamplesOpen(opener));
    examples
}

/// The example cards over the screen while open, mounted fresh per tag.
#[component]
pub fn OtagExamplesHost(examples: OtagExamples) -> Element {
    let OtagExamples { open, slug } = examples;
    rsx! {
        if open() {
            for tag in [slug()] {
                OracleTagExamples { key: "{tag}", open, slug: tag }
            }
        }
    }
}
