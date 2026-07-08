use crate::WEB_BASE;
use dioxus::prelude::*;
use zwipe_components::{PageMeta as SharedPageMeta, SiteMeta};

/// Zwipe's site constants for the shared head-meta component.
const SITE: SiteMeta = SiteMeta {
    base_url: WEB_BASE,
    site_name: "Zwipe",
    og_image_path: Some("/assets/og-default.png"),
};

/// Thin wrapper over the shared [`SharedPageMeta`]: bakes in the site config
/// so pages keep calling `PageMeta { title, description, path }` unchanged.
#[component]
pub fn PageMeta(title: String, description: String, path: String) -> Element {
    rsx! {
        SharedPageMeta { site: SITE, title, description, path }
    }
}
