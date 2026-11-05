//! The deck view on a wide window: the deck page both sites share, with the
//! owner's pieces in its slots. The head carries Edit, Clone, Share and the
//! page's `?`; the cards are the deck's rows; the stats fold under them;
//! and the foot holds Warnings, Data and Danger zone.

use std::collections::{HashMap, HashSet};

use components::{
    DeckBand, DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage, FeaturedCard, sections,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckProfile, DeckWarning, deck_tag_label};
use zwipe_ui::deck_cards::{self, Zone};

use super::cards::zone;
use super::dialogs::DeckDialogs;
use super::panels::{DangerZonePanel, DataPanel};
use super::sections::{
    BudgetBody, CurrencyChips, DistributionsBody, DrawOddsStepper, ManaBody, ProfileActions,
    TagsBody, WarningFixes, WarningsBody, charted,
};
use crate::domain::deck::{has_budget, tag_count};
use crate::inbound::cards::card_row::{CardOverlays, CardRow, ShowRowArt};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::screens::deck::skeletons::DeckCardsSkeleton;

/// The stat panels' fold keys, shut when the page opens.
const FOLDED_STATS: [&str; 5] = ["Budget", "Tags", "Distributions", "Mana", "Draw odds"];

/// The page for a loaded `profile`; `deck` arrives after it and brings the
/// cards, the metrics and the warnings.
#[component]
pub fn DeckWidePage(
    deck_id: Uuid,
    profile: DeckProfile,
    deck: Option<Deck>,
    deck_loading: bool,
    metrics: Option<DeckMetrics>,
    buy: Option<(String, String)>,
    currency: Signal<PriceCurrency>,
    warnings: Vec<DeckWarning>,
    fixes: WarningFixes,
    dialogs: DeckDialogs,
    on_hint: Callback<()>,
) -> Element {
    let group_by = use_signal(|| GroupByOption::CardType);
    let show_zone = use_signal(|| true);
    let show_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_art));
    let name = use_signal(String::new);
    let types = use_signal(Vec::<CardType>::new);
    let colors = use_signal(Vec::<Color>::new);
    let folded: Signal<HashSet<String>> =
        use_signal(|| FOLDED_STATS.into_iter().map(String::from).collect());
    let expanded: Signal<Option<Uuid>> = use_signal(|| None);
    let overlays = try_use_context::<CardOverlays>();

    // The command zone in its slots, the main board without it, and what
    // each card's row shows.
    let held = deck.as_ref().map_or_else(Zone::default, zone);
    let mut entries = deck.as_ref().map(|d| d.entries.clone()).unwrap_or_default();
    held.strip(&mut entries);
    let qty: HashMap<Uuid, i32> = entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .map(|e| (e.card.scryfall_data.id, *e.deck_card.quantity))
        .collect();
    let mvp: HashSet<Uuid> = entries
        .iter()
        .filter(|e| e.deck_card.mvp_at.is_some())
        .map(|e| e.card.scryfall_data.id)
        .collect();
    let oathbreaker = profile.format.is_some_and(|f| f.has_signature_spell());
    let featured = deck_cards::featured(&held, &entries, oathbreaker);

    let mut builder = CardQueryBuilder::new();
    let query = name();
    if !query.trim().is_empty() {
        builder.set_name_contains(query.trim());
    }
    if !types().is_empty() {
        builder.set_card_type_contains_any(types());
    }
    if !colors().is_empty() {
        builder.set_color_identity_within(colors().into_iter().collect());
    }
    let listed = if show_zone() {
        held.clone()
    } else {
        Zone::default()
    };
    let display = deck_cards::pass(&entries, &listed, &builder, group_by());
    let bands = sections(&display, profile.format, None, &qty);
    let empty = deck.is_some() && bands.is_empty();

    let total_cards = metrics.as_ref().map_or_else(
        || profile.card_count,
        |m| i64::try_from(m.total_cards).unwrap_or(profile.card_count),
    );
    let shown = currency();
    let price = metrics.as_ref().and_then(|m| match shown {
        PriceCurrency::Eur => m.total_price_eur,
        PriceCurrency::Tix => m.total_price_tix,
        PriceCurrency::Usd => m.total_price_usd,
    });
    let facts = DeckFacts {
        cards: total_cards,
        format: profile.format,
        power_level: profile.power_level,
        zone_names: [
            profile.commander_name.clone(),
            profile.partner_commander_name.clone(),
            profile.background_name.clone(),
            profile.signature_spell_name.clone(),
        ]
        .into_iter()
        .flatten()
        .collect(),
        tags: profile.tags.iter().map(|t| deck_tag_label(t)).collect(),
        other_tags: profile
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
        price: price.filter(|p| *p > 0.0).map(|p| shown.format_amount(p)),
    };
    let charts = charted(metrics.as_ref());
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    let has_cards = metrics.is_some();

    rsx! {
        DeckPage {
            head: rsx! {
                DeckHead {
                    name: profile.name.to_string(),
                    facts,
                    has_featured: !featured.is_empty(),
                    actions: rsx! {
                        ProfileActions { deck_id, dialogs, on_hint }
                    },
                    for (card, role) in featured.iter().cloned() {
                        FeaturedCard {
                            key: "{card.scryfall_data.id}-{role}",
                            card,
                            role: role.to_string(),
                            on_open: overlays.map(|CardOverlays { mut preview, .. }| {
                                Callback::new(move |(sd, face)| preview.set(Some((sd, face))))
                            }),
                        }
                    }
                }
            },
            controls: rsx! {
                DeckControls { group_by, show_zone, show_art, name, types, colors }
            },
            empty,
            bands: rsx! {
                if deck.is_none() && deck_loading {
                    DeckCardsSkeleton {}
                }
                for section in bands {
                    DeckBand {
                        key: "{section.key}",
                        section: section.clone(),
                        folded,
                        for card in section.cards {
                            CardRow {
                                key: "{card.scryfall_data.id}",
                                qty: qty.get(&card.scryfall_data.id).copied().unwrap_or(1),
                                mvp: mvp.contains(&card.scryfall_data.id).then_some(true),
                                card,
                                expanded_card: expanded,
                                price_currency: shown,
                            }
                        }
                    }
                }
            },
            stats: rsx! {
                if has_budget(&profile, has_cards) {
                    DeckFold {
                        fold_key: "Budget",
                        title: "Budget",
                        folded,
                        accessory: has_cards.then(|| rsx! { CurrencyChips { currency } }),
                        BudgetBody { profile: profile.clone(), metrics, buy, currency }
                    }
                }
                if tag_count(&profile) > 0 {
                    DeckFold { fold_key: "Tags", title: "Tags", folded,
                        TagsBody { profile: profile.clone() }
                    }
                }
                if let Some(m) = charts.clone() {
                    DeckFold { fold_key: "Distributions", title: "Distributions", folded,
                        DistributionsBody { metrics: m }
                    }
                }
                if charts.is_some() || profile.land_target.is_some() {
                    DeckFold { fold_key: "Mana", title: "Mana", folded,
                        ManaBody { charts: charts.clone(), land_count, land_target: profile.land_target }
                    }
                }
                if let Some(m) = charts {
                    DeckFold {
                        fold_key: "Draw odds",
                        title: "Draw odds",
                        folded,
                        accessory: rsx! { InfoButton { topic: HintTopic::DrawOdds } },
                        DrawOddsStepper { metrics: m }
                    }
                }
            },
            foot: rsx! {
                div { class: "web-column",
                    if !warnings.is_empty() {
                        div { class: "web-section open warn web-deck-warnings",
                            div { class: "web-section-head",
                                div { class: "web-section-title",
                                    span { class: "settings-list-title", "Warnings" }
                                    span { class: "web-section-badge", "{warnings.len()}" }
                                }
                            }
                            hr { class: "web-section-rule" }
                            WarningsBody { deck_id, warnings, fixes }
                        }
                    }
                    DataPanel { deck_id, has_cards }
                }
                DangerZonePanel { dialogs }
            },
        }
    }
}
