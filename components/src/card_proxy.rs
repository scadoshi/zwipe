//! The card a missing or failed image leaves: the card's own text laid out
//! like its expanded row, in the exact box the image would fill.
//!
//! [`CardProxy`] is always the card's 63:88 shape, so nothing around it moves
//! when an image is missing. Top to bottom: the name and mana cost, a rule,
//! the row's chips (type line, rarity, set, then any roles), the rules text,
//! and power and toughness (or loyalty) as a chip after the text. The text is
//! sized against the box, and [`use_card_proxy_fit`] shrinks a card whose text
//! would still overflow until it fits. At a thumbnail's size only the name
//! shows.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

use crate::oracle_text::OracleText;

/// What a [`CardProxy`] shows for one face of a card.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyFace {
    /// The face's name.
    pub name: String,
    /// The face's mana cost in Scryfall's `{R}{1}` form, when it has one.
    pub mana_cost: Option<String>,
    /// The face's type line.
    pub type_line: Option<String>,
    /// The card's rarity, as its row names it.
    pub rarity: Option<String>,
    /// The card's set name.
    pub set_name: Option<String>,
    /// The face's rules text.
    pub oracle_text: Option<String>,
    /// Power and toughness as `2/2`, or loyalty, when the face has either.
    pub stats: Option<String>,
}

impl ProxyFace {
    /// Face `face` of `sd` (the front when it has no such face): the face's own
    /// name, cost, type, text and stats where it carries them, else the
    /// card's. Rarity and set are the card's.
    #[must_use]
    pub fn of(sd: &ScryfallData, face: usize) -> Self {
        let side = sd.card_faces.as_ref().and_then(|faces| faces.get(face));
        let pick = |own: Option<&String>, card: Option<&String>| own.or(card).cloned();
        let power = pick(side.and_then(|f| f.power.as_ref()), sd.power.as_ref());
        let toughness = pick(
            side.and_then(|f| f.toughness.as_ref()),
            sd.toughness.as_ref(),
        );
        let loyalty = pick(side.and_then(|f| f.loyalty.as_ref()), sd.loyalty.as_ref());
        let stats = match (power, toughness) {
            (Some(p), Some(t)) => Some(format!("{p}/{t}")),
            _ => loyalty,
        };
        Self {
            name: side.map_or_else(|| sd.name.clone(), |f| f.name.clone()),
            mana_cost: pick(side.map(|f| &f.mana_cost), sd.mana_cost.as_ref())
                .filter(|cost| !cost.is_empty()),
            type_line: pick(
                side.and_then(|f| f.type_line.as_ref()),
                sd.type_line.as_ref(),
            ),
            rarity: Some(sd.rarity.to_long_name()),
            set_name: Some(sd.set_name.clone()),
            oracle_text: pick(
                side.and_then(|f| f.oracle_text.as_ref()),
                sd.oracle_text.as_ref(),
            ),
            stats,
        }
    }
}

/// The no-image card for `face`: its name and cost, a rule, its chips (with
/// `roles` after the set, as the expanded row shows them), its rules text and
/// its stats chip, in the card's 63:88 box. `trailing` sits at the head's
/// right end, before the cost, for a host's control such as Flip.
#[component]
pub fn CardProxy(
    face: ProxyFace,
    #[props(default)] roles: Vec<String>,
    #[props(default)] trailing: Option<Element>,
    #[props(default = String::new())] class: String,
) -> Element {
    let ProxyFace {
        name,
        mana_cost,
        type_line,
        rarity,
        set_name,
        oracle_text,
        stats,
    } = face;
    rsx! {
        div { class: "card-proxy {class}", role: "img", "aria-label": "{name}",
            div { class: "card-proxy-fit",
                div { class: "card-proxy-head",
                    span { class: "card-proxy-name", "{name}" }
                    span { class: "card-proxy-head-end",
                        {trailing}
                        if let Some(cost) = mana_cost {
                            OracleText { text: cost, class: "card-proxy-cost".to_string() }
                        }
                    }
                }
                hr { class: "card-proxy-rule" }
                div { class: "card-proxy-chips",
                    if let Some(line) = type_line {
                        span { class: "detail-chip detail-chip-type", "{line}" }
                    }
                    if let Some(rarity) = rarity {
                        span { class: "detail-chip detail-chip-rarity", "{rarity}" }
                    }
                    if let Some(set) = set_name {
                        span { class: "detail-chip detail-chip-set", "{set}" }
                    }
                    for role in roles {
                        span { key: "{role}", class: "stat-chip", "{role}" }
                    }
                }
                if let Some(text) = oracle_text {
                    OracleText { text, class: "card-proxy-text".to_string() }
                }
                if let Some(stats) = stats {
                    span { class: "detail-chip detail-chip-pt card-proxy-stats", "{stats}" }
                }
            }
        }
    }
}

/// Shrinks every [`CardProxy`] on the page whose text overflows its box, and
/// again whenever one is added or resized: its text scale steps down from 1
/// until the content fits, to no less than 0.45 of the box-sized type.
pub const FIT_JS: &str = r"
(() => {
  if (window.__zwProxyFit) return;
  window.__zwProxyFit = true;
  const fit = (card) => {
    const inner = card.querySelector('.card-proxy-fit');
    if (!inner || card.clientWidth <= 110) return;
    let scale = 1;
    card.style.setProperty('--proxy-scale', '1');
    while (inner.scrollHeight > card.clientHeight + 1 && scale > 0.45) {
      scale = Math.round((scale - 0.05) * 100) / 100;
      card.style.setProperty('--proxy-scale', String(scale));
    }
  };
  const sizes = new ResizeObserver((entries) => { for (const e of entries) fit(e.target); });
  const watch = (root) => {
    const cards = root.matches && root.matches('.card-proxy') ? [root] : [];
    if (root.querySelectorAll) cards.push(...root.querySelectorAll('.card-proxy'));
    for (const card of cards) { sizes.observe(card); fit(card); }
  };
  watch(document.body);
  new MutationObserver((records) => {
    for (const r of records) for (const n of r.addedNodes) if (n instanceof Element) watch(n);
  }).observe(document.body, { childList: true, subtree: true });
})();
";

/// Installs the fitting for every [`CardProxy`] on the page, once per document.
pub fn use_card_proxy_fit() {
    use_hook(|| {
        document::eval(FIT_JS);
    });
}
