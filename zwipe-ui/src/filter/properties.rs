//! Property tests for the filter sheet's draft: what Apply and Reset do and what the toast says,
//! for any filter on any host the clients open the sheet on, and the forms a filter is kept
//! and sent in.

use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::card_filter::query::CardQuery;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::Format;

use super::AddContext;
use super::draft::{ApplyOutcome, FilterDraft, ResetRule, SheetConfig};
use crate::screens::card_filter::as_sent;
use crate::stores::filters::{FilterScope, FilterStore};

/// 256 cases from a fixed seed, so a run is the same everywhere; failures are kept in
/// `proptest-regressions/` and replayed first.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(592),
        ..ProptestConfig::default()
    }
}

/// Some of `items`, possibly none, in their order.
fn some_of<T: Clone + std::fmt::Debug>(items: Vec<T>) -> impl Strategy<Value = Vec<T>> {
    let len = items.len();
    prop::collection::vec(any::<bool>(), len).prop_map(move |keep| {
        items
            .iter()
            .zip(keep)
            .filter(|(_, keep)| *keep)
            .map(|(item, _)| item.clone())
            .collect()
    })
}

/// A format the decks in these tests use.
fn format() -> impl Strategy<Value = Format> {
    prop::sample::select(vec![Format::Commander, Format::Modern, Format::Pauper])
}

/// How often [`filter`] sets each search criterion: rarely enough that about a quarter of the
/// filters set none, so blank filters and ones holding only a mode or a currency come up often.
const SET: f64 = 0.15;

/// A filter as the sheet edits one: any of a name, a mana value or range, a price range, a
/// color identity, types to include and exclude, legalities, a sort, synergy, "In my
/// collection" and a currency, so blank and default filters and ones that include and exclude
/// the same type all come up.
fn filter() -> impl Strategy<Value = CardQueryBuilder> {
    let text = (
        prop::option::weighted(SET, "[ab]{1,3}"),
        prop::option::weighted(SET, 0u8..=6),
        prop::option::weighted(SET, (0u8..=6, 0u8..=6)),
        prop::option::weighted(SET, 0u8..=20),
        prop::option::weighted(SET, 0u8..=20),
    );
    let picks = (
        prop::option::weighted(SET, some_of(vec![Color::White, Color::Red, Color::Green])),
        prop::option::weighted(
            SET,
            some_of(vec![CardType::Creature, CardType::Land, CardType::Artifact]),
        ),
        prop::option::weighted(SET, some_of(vec![CardType::Land, CardType::Instant])),
        prop::option::weighted(SET, some_of(vec!["commander", "modern", "pauper"])),
    );
    let modes = (
        prop::option::of(prop::sample::select(vec![
            CardSortKey::Name,
            CardSortKey::Cmc,
            CardSortKey::PriceUsd,
        ])),
        any::<bool>(),
        any::<bool>(),
        prop::option::of(prop::sample::select(vec![
            PriceCurrency::Usd,
            PriceCurrency::Eur,
        ])),
    );
    (text, picks, modes).prop_map(|(text, picks, modes)| {
        let (name, cmc, range, min, max) = text;
        let (colors, types, excluded, legal) = picks;
        let (sort, synergy, owned, currency) = modes;
        let mut b = CardQueryBuilder::default();
        if let Some(name) = name {
            b.set_name_contains(name);
        }
        if let Some(cmc) = cmc {
            b.set_cmc_equals(f64::from(cmc));
        }
        if let Some((x, y)) = range {
            b.set_cmc_range((f64::from(x.min(y)), f64::from(x.max(y))));
        }
        if let Some(min) = min {
            b.set_price_min(f64::from(min));
        }
        if let Some(max) = max {
            b.set_price_max(f64::from(max));
        }
        if let Some(colors) = colors {
            b.set_color_identity_within(Colors::from(colors));
        }
        if let Some(types) = types {
            b.set_card_type_contains_any(types);
        }
        if let Some(types) = excluded {
            b.set_card_type_excludes_any(types);
        }
        if let Some(legal) = legal {
            b.set_legalities_contains_any(legal);
        }
        if let Some(sort) = sort {
            b.set_sort(sort);
        }
        if let Some(currency) = currency {
            b.set_price_currency(currency);
        }
        b.set_synergy(synergy);
        b.set_owned_only(owned);
        b
    })
}

/// The deck facts Add's search is built from.
fn add_context() -> impl Strategy<Value = AddContext> {
    (
        prop::option::of(format()),
        prop::option::of(some_of(vec![Color::Blue, Color::Red, Color::Green])),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(format, colors, has_commander, lands_at_target)| AddContext {
                format,
                colors: colors.map(Colors::from),
                has_commander,
                lands_at_target,
            },
        )
}

/// A host as the clients open the sheet: a plain host (the deck cards view, Remove, the
/// commander maybeboard, Swipe select), Add's search or Add's maybeboard, each with the
/// settings that host passes, "In my collection" offered or not where the host can offer it.
fn host() -> impl Strategy<Value = SheetConfig> {
    prop_oneof![
        any::<bool>().prop_map(|show_owned| SheetConfig {
            show_owned,
            ..SheetConfig::default()
        }),
        (add_context(), any::<bool>()).prop_map(|(context, show_owned)| SheetConfig {
            show_format: true,
            validate: true,
            deck_format: context.format,
            reset: ResetRule::AddSearch(context),
            show_owned,
            ..SheetConfig::default()
        }),
        Just(SheetConfig {
            show_format: true,
            reset: ResetRule::AddMaybeboard,
            ..SheetConfig::default()
        }),
    ]
}

/// The filter Reset gives on `config`'s host from `filter`, written out from the rules rather
/// than by calling them: every host keeps the price currency; a plain host goes back to blank;
/// Add's search to its deck context with synergy kept; Add's maybeboard to blank with synergy
/// kept. Add leaves a filter alone that sets nothing beyond that default and has no sort.
fn host_default(config: &SheetConfig, filter: &CardQueryBuilder) -> CardQueryBuilder {
    let mut b = CardQueryBuilder::default();
    if let Some(currency) = filter.price_currency() {
        b.set_price_currency(currency);
    }
    match &config.reset {
        ResetRule::Plain => {}
        ResetRule::AddSearch(context) => {
            let default = filter.is_empty_ignoring_deck_context_and_auto_lands(
                context.format,
                context.lands_at_target,
            ) && filter.sort().is_none();
            if default {
                return filter.clone();
            }
            context.apply_deck_context(&mut b);
            b.set_synergy(filter.synergy());
        }
        ResetRule::AddMaybeboard => {
            if filter.is_empty() && filter.sort().is_none() {
                return filter.clone();
            }
            b.set_synergy(filter.synergy());
        }
    }
    b
}

/// The query as the server reads it, or `None` for a filter that doesn't build.
fn wire(filter: &CardQueryBuilder) -> Option<String> {
    filter
        .build()
        .ok()
        .and_then(|query| serde_json::to_string(&query).ok())
}

proptest! {
    #![proptest_config(config())]

    /// Applying is idempotent: the same draft gives the same outcome and sends the same query
    /// twice, and a sheet reopened over what was applied has nothing to apply.
    #[test]
    fn apply_is_idempotent(config in host(), snapshot in filter(), draft in filter()) {
        let mut sheet = FilterDraft::new(&snapshot, config.clone());
        sheet.draft = draft;
        let first = sheet.apply();
        prop_assert_eq!(&first, &sheet.apply());
        if let ApplyOutcome::Applied { filter, .. } = first {
            let sent = as_sent(&filter, config.show_owned);
            prop_assert_eq!(wire(&sent), wire(&as_sent(&filter, config.show_owned)));
            let reopened = FilterDraft::new(&filter, config);
            prop_assert_eq!(reopened.apply(), ApplyOutcome::Disabled);
        }
    }

    /// Reset stages the host's default, a second Reset has nothing to do, and Apply then
    /// commits exactly that default, unless it is what was already applied or Add's search
    /// refuses to serve everything.
    #[test]
    fn reset_then_apply_lands_on_the_hosts_default(
        config in host(),
        snapshot in filter(),
        draft in filter(),
    ) {
        let mut sheet = FilterDraft::new(&snapshot, config.clone());
        sheet.draft = draft.clone();
        let _ = sheet.reset();
        let staged = sheet.draft.clone();
        prop_assert_eq!(&staged, &host_default(&config, &draft));

        let second = sheet.reset();
        prop_assert_eq!(second.text.as_str(), "Filter already at default");
        prop_assert_eq!(&sheet.draft, &staged);

        match sheet.apply() {
            ApplyOutcome::Applied { filter, .. } => prop_assert_eq!(*filter, staged),
            ApplyOutcome::Disabled => prop_assert_eq!(&staged, sheet.snapshot()),
            ApplyOutcome::Refused(toast) => {
                prop_assert!(config.validate, "only Add's search refuses: {}", toast.text);
            }
        }
    }

    /// "In my collection" reaches the server only from a host that offers it, and dropping it
    /// changes nothing else. A host that doesn't offer it opens with it off in the draft and in
    /// what Apply compares with, so whatever the sheet applies doesn't carry it.
    #[test]
    fn a_hidden_owned_flag_is_never_sent(
        config in host(),
        snapshot in filter(),
        draft in filter(),
        offered in any::<bool>(),
    ) {
        let sent = as_sent(&draft, offered);
        prop_assert_eq!(sent.owned_only(), draft.owned_only() && offered);
        let mut restored = sent.clone();
        restored.set_owned_only(draft.owned_only());
        prop_assert_eq!(&restored, &draft);
        if !offered && let Some(json) = wire(&sent) {
            prop_assert!(!json.contains("owned_only"), "{}", json);
        }

        let mut sheet = FilterDraft::new(&snapshot, config.clone());
        if !config.show_owned {
            prop_assert!(!sheet.draft.owned_only());
            prop_assert!(!sheet.snapshot().owned_only());
            sheet.draft = as_sent(&draft, false);
            if let ApplyOutcome::Applied { filter, .. } = sheet.apply() {
                prop_assert!(!filter.owned_only());
            }
        }
    }

    /// A filter comes back the same from every form it is kept or sent in: its own JSON, the
    /// query's JSON, and the store a screen parks it in.
    #[test]
    fn a_filter_round_trips_losslessly(draft in filter(), other in filter()) {
        let json = serde_json::to_string(&draft).unwrap();
        prop_assert_eq!(&serde_json::from_str::<CardQueryBuilder>(&json).unwrap(), &draft);

        if let Ok(query) = draft.build() {
            let json = serde_json::to_string(&query).unwrap();
            prop_assert_eq!(serde_json::from_str::<CardQuery>(&json).unwrap(), query);
        }

        let (deck, elsewhere) = (Uuid::from_u128(1), Uuid::from_u128(2));
        let mut store = FilterStore::default();
        store.park(FilterScope::Cards, elsewhere, other.clone());
        store.park(FilterScope::Remove, deck, other.clone());
        store.park(FilterScope::Cards, deck, draft.clone());
        prop_assert_eq!(store.restore(FilterScope::Cards, deck), Some(draft));
        prop_assert_eq!(store.restore(FilterScope::Cards, elsewhere), Some(other.clone()));
        prop_assert_eq!(store.restore(FilterScope::Remove, deck), Some(other));
        prop_assert_eq!(store.restore(FilterScope::AddSearch, deck), None);
    }

    /// Apply's toast says Filter cleared exactly when it commits what Reset would give from the
    /// filter that was applied, and Filter applied otherwise, for any draft and for one Reset
    /// just staged.
    #[test]
    fn the_toast_says_cleared_exactly_at_the_default(
        config in host(),
        snapshot in filter(),
        draft in filter(),
        reset_first in any::<bool>(),
    ) {
        let mut sheet = FilterDraft::new(&snapshot, config.clone());
        sheet.draft = draft;
        if reset_first {
            let _ = sheet.reset();
        }
        if let ApplyOutcome::Applied { filter, toast } = sheet.apply() {
            let at_default = *filter == host_default(&config, sheet.snapshot());
            let want = if at_default { "Filter cleared" } else { "Filter applied" };
            prop_assert_eq!(toast.text.as_str(), want, "{:?}", config.reset);
        }
    }
}
