//! One-time hints: a dialog that opens by itself the first time a user
//! reaches a screen, on any device, and never again.
//!
//! Whether a user has seen one lives on the server, in the user's
//! `hints_shown` map. The dialog opens at once and the hint is recorded with
//! `mark_hint_shown` in the background; a failed record only means it may
//! open once more later. A screen's header `?` reopens it on demand without
//! recording it again.

use components::{HintBullet, HintBullets, HintDialog, HintKey, HintLine};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints;

use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::session_store::SessionStore;

/// Every one-time hint, keyed as the server records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneTimeHint {
    /// Home, on mount.
    FirstLogin,
    /// Account, on mount.
    Profile,
    /// Config, on mount.
    Config,
    /// The first time the Exceptions sheet opens in a Config visit.
    UniversesBeyondExceptions,
    /// The deck list, once it has a deck.
    DeckList,
    /// Create deck, on mount.
    CreateDeck,
    /// Edit deck, on mount.
    EditDeck,
    /// Deck view, on mount.
    FirstDeck,
    /// Deck cards, once the deck has entries.
    DeckCards,
    /// Add cards, on mount.
    AddDeckCards,
    /// Remove cards, on mount.
    RemoveDeckCards,
    /// The first use of a filter section on a screen. Its dialog is
    /// [`FilterHint`].
    Filter,
    /// Import, on mount.
    Import,
    /// Export, on mount.
    Export,
    /// The oracle tag dictionary, on mount.
    OracleTagDictionary,
    /// Oracle tag examples, on mount.
    OtagExamples,
    /// Commander maybeboard, on mount.
    CommanderMaybeboard,
    /// Playstyle, on mount.
    SuggestedCommanders,
    /// Activity, on mount.
    Activity,
    /// The first time Swipe select opens on a screen. Its dialog is
    /// [`SwipeSelectHint`], which depends on the mode.
    SwipeSelect,
}

impl OneTimeHint {
    /// The key the server records the hint under.
    pub fn key(self) -> &'static str {
        match self {
            Self::FirstLogin => hints::HINT_FIRST_LOGIN,
            Self::Profile => hints::HINT_PROFILE,
            Self::Config => hints::HINT_CONFIG,
            Self::UniversesBeyondExceptions => hints::HINT_UNIVERSES_BEYOND_EXCEPTIONS,
            Self::DeckList => hints::HINT_DECK_LIST,
            Self::CreateDeck => hints::HINT_CREATE_DECK,
            Self::EditDeck => hints::HINT_EDIT_DECK,
            Self::FirstDeck => hints::HINT_FIRST_DECK,
            Self::DeckCards => hints::HINT_DECK_CARDS,
            Self::AddDeckCards => hints::HINT_ADD_DECK_CARDS,
            Self::RemoveDeckCards => hints::HINT_REMOVE_DECK_CARDS,
            Self::Filter => hints::HINT_FILTER,
            Self::Import => hints::HINT_IMPORT,
            Self::Export => hints::HINT_EXPORT,
            Self::OracleTagDictionary => hints::HINT_ORACLE_TAG_DICTIONARY,
            Self::OtagExamples => hints::HINT_OTAG_EXAMPLES,
            Self::CommanderMaybeboard => hints::HINT_COMMANDER_MAYBEBOARD,
            Self::SuggestedCommanders => hints::HINT_SUGGESTED_COMMANDERS,
            Self::Activity => hints::HINT_ACTIVITY,
            Self::SwipeSelect => hints::HINT_SWIPE_SELECT,
        }
    }
}

/// Opens the returned signal at mount, once per account, and records the
/// hint. For a hint gated on something that loads later, call
/// [`open_and_record_hint`] once the gate first passes instead.
pub fn use_one_time_hint(hint: OneTimeHint) -> Signal<bool> {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let open = use_signal(|| false);
    use_hook(move || open_and_record_hint(hint, session, client, open));
    open
}

/// Opens `open` and records `hint` as shown, unless there is no session or
/// the user has seen it. A caller with a gate guards it to run once.
pub fn open_and_record_hint(
    hint: OneTimeHint,
    mut session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    mut open: Signal<bool>,
) {
    let key = hint.key();
    let seen = session
        .peek()
        .as_ref()
        .is_none_or(|s| s.user.has_seen_hint(key));
    if seen {
        return;
    }
    open.set(true);
    spawn(async move {
        let Ok(current) = ensure_fresh(session, client).await else {
            return;
        };
        let http = client.peek().clone();
        match http.mark_hint_shown(key, &current).await {
            Ok(user) => {
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.user = user;
                    SessionStore::browser().infallible_save(&latest);
                    session.set(Some(latest));
                }
            }
            Err(e) => tracing::warn!("failed to record hint {key}: {e}"),
        }
    });
}

/// A colored word inside a hint, such as a swipe direction. `color` names a
/// CSS variable, like `--color-success`.
#[component]
pub fn HintColored(color: String, children: Element) -> Element {
    rsx! {
        span { style: "color: var({color}); font-weight: 600;", {children} }
    }
}

/// The dialog for every one-time hint whose copy is fixed, which is all of
/// them but [`OneTimeHint::Filter`] and [`OneTimeHint::SwipeSelect`]; those
/// two draw nothing here.
#[component]
pub fn OneTimeHintDialog(hint: OneTimeHint, open: Signal<bool>) -> Element {
    let Some(title) = fixed_title(hint) else {
        return rsx! {};
    };
    rsx! {
        HintDialog { open, title, {fixed_body(hint)} }
    }
}

/// The title of a hint with fixed copy.
fn fixed_title(hint: OneTimeHint) -> Option<&'static str> {
    Some(match hint {
        OneTimeHint::FirstLogin => "Welcome to Zwipe",
        OneTimeHint::Profile => "Your account",
        OneTimeHint::Config => "Config",
        OneTimeHint::UniversesBeyondExceptions => "Franchise exceptions",
        OneTimeHint::DeckList => "Your decks",
        OneTimeHint::CreateDeck | OneTimeHint::EditDeck => "Building a deck",
        OneTimeHint::FirstDeck => "Welcome to your deck",
        OneTimeHint::DeckCards => "Browsing your deck",
        OneTimeHint::AddDeckCards => "Swipe to build",
        OneTimeHint::RemoveDeckCards => "Swipe to trim",
        OneTimeHint::Import => "Importing cards",
        OneTimeHint::Export => "Exporting your deck",
        OneTimeHint::OracleTagDictionary => "Oracle tags",
        OneTimeHint::OtagExamples => "Example cards",
        OneTimeHint::CommanderMaybeboard => "Commander maybeboard",
        OneTimeHint::SuggestedCommanders => "Playstyle",
        OneTimeHint::Activity => "Activity",
        OneTimeHint::Filter | OneTimeHint::SwipeSelect => return None,
    })
}

/// The body of a hint with fixed copy, as the app words it.
#[expect(
    clippy::too_many_lines,
    reason = "a lookup table with one short arm per hint"
)]
fn fixed_body(hint: OneTimeHint) -> Element {
    match hint {
        OneTimeHint::FirstLogin => rsx! {
            HintBullets {
                HintBullet { HintKey { "Decks" } " holds your decks" }
                HintBullet { HintKey { "Color map" } " grows from your decks' colors. Tap Playstyle for suggested commanders" }
                HintBullet { HintKey { "Account" } " has your email, password and log out" }
                HintBullet { HintKey { "Config" } " sets your theme and which cards show" }
                HintBullet { HintColored { color: "--color-warning", "Verify your email" } " in Account for full deck and card limits" }
            }
        },
        OneTimeHint::Profile => rsx! {
            HintBullets {
                HintBullet { "Tap a row's " HintKey { color: "--palette-6", "?" } " for what it does" }
                HintBullet { HintKey { "More" } " deletes your account" }
            }
        },
        OneTimeHint::Config => rsx! {
            HintBullets {
                HintBullet { "Set how Zwipe looks and which cards it shows" }
                HintBullet { "Tap a row's " HintKey { color: "--palette-6", "?" } " for what it does" }
            }
        },
        OneTimeHint::UniversesBeyondExceptions => rsx! {
            HintLine { "Picked franchises still show while Universes Beyond is hidden" }
        },
        OneTimeHint::DeckList => rsx! {
            HintBullets {
                HintBullet { "Tap a deck to open it" }
                HintBullet { HintKey { "Group by" } " folds the list into sections, " HintKey { "Show" } " narrows it by color or tag" }
                HintBullet { HintKey { "Create" } " starts a new deck" }
            }
        },
        OneTimeHint::CreateDeck | OneTimeHint::EditDeck => rsx! {
            HintBullets {
                HintBullet { HintColored { color: "--accent-primary", "Profile" } " is what the deck is, " HintColored { color: "--accent-secondary", "Budget" } " what it may cost, " HintColored { color: "--accent-tertiary", "Tags" } " what it does" }
                HintBullet { "Tags shape which cards we suggest" }
                HintBullet { "Tap a field's " HintKey { color: "--palette-6", "?" } " for what it does" }
            }
        },
        OneTimeHint::FirstDeck => rsx! {
            HintBullets {
                HintBullet { HintKey { "Cards" } " browses, adds and removes cards" }
                HintBullet { HintKey { "Edit" } " changes the name, format, command zone, tags and targets" }
                HintBullet { HintKey { "More" } " imports, exports, clones, shares or deletes the deck" }
                HintBullet { "Skipped and removed cards stay out of the add stack. " HintKey { "More" } " then Clear skips brings them back" }
                HintBullet { "Stats fill in as the deck grows. Tap a section to expand it" }
                HintBullet { "Warnings flag rule problems, each with a one-tap fix" }
            }
        },
        OneTimeHint::DeckCards => rsx! {
            HintLine { "Tap a row for details, quantity and printings" }
            HintBullets {
                HintBullet { "Tap a " HintColored { color: "--palette-1", "keyword" } " chip for what it does, or a " HintColored { color: "--palette-1", "role" } " chip for its oracle tags" }
                HintBullet { HintKey { "Group by" } " sorts by type, mana value, color or role" }
                HintBullet { HintKey { "Boards" } " picks main, maybe or side" }
                HintBullet { HintKey { "Show" } " toggles tokens, the command zone and row art" }
                HintBullet { HintKey { color: "--color-warning", "Star" } " marks up to three MVPs. Suggestions lean toward cards that share their roles" }
            }
        },
        OneTimeHint::AddDeckCards => rsx! {
            HintBullets {
                HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " adds the card" }
                HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it for a week" }
                HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " sends it to the maybeboard" }
                HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
                HintBullet { HintKey { "Synergy" } " on starts with cards that work with your commander, then cards like your deck. Off shows every legal card" }
            }
            HintLine { "Sort only reorders. It never hides cards" }
        },
        OneTimeHint::RemoveDeckCards => rsx! {
            HintBullets {
                HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " removes the card" }
                HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " keeps it" }
                HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " moves it between main and maybeboard" }
                HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
            }
            HintLine { "The board chips up top pick which board you are trimming" }
        },
        OneTimeHint::Import => rsx! {
            HintBullets {
                HintBullet { "Paste a decklist, or switch " HintKey { "From" } " to an Archidekt deck link" }
                HintBullet { HintKey { "Add" } " keeps your current cards. " HintKey { color: "--color-warning", "Replace" } " swaps the board's contents" }
                HintBullet { HintKey { "Board" } " picks where the cards go" }
                HintBullet { "Importing clears the deck's undo history" }
            }
        },
        OneTimeHint::Export => rsx! {
            HintBullets {
                HintBullet { HintKey { "Export" } " picks which boards to include" }
                HintBullet { HintKey { "Copy" } " puts the decklist on your clipboard" }
            }
        },
        OneTimeHint::OracleTagDictionary => rsx! {
            HintBullets {
                HintBullet { "Tap a " HintKey { color: "--palette-1", "letter" } " to browse, or " HintColored { color: "--accent-secondary", "search" } " by name or description" }
                HintBullet { HintKey { "Examples" } " shows real cards with that tag" }
                HintBullet { HintKey { "Use" } " adds the tag where you came from" }
                HintBullet { "Some tags have no description yet" }
            }
        },
        OneTimeHint::OtagExamples => rsx! {
            HintBullets {
                HintBullet { "Swipe " HintColored { color: "--color-success", "left" } " for the next card" }
                HintBullet { "Swipe " HintColored { color: "--accent-secondary", "down" } " to go back one" }
            }
        },
        OneTimeHint::CommanderMaybeboard => rsx! {
            HintLine { "Commanders you save land here" }
            HintBullets {
                HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " while picking a commander saves it here" }
                HintBullet { HintKey { "Swipe" } " deals commanders: right starts a deck, up saves one" }
                HintBullet { HintKey { "Quick add" } " searches by name" }
                HintBullet { HintKey { "Create deck" } " starts a deck with that commander" }
                HintBullet { HintKey { color: "--color-error", "Remove" } " takes it off the list" }
            }
        },
        OneTimeHint::SuggestedCommanders => rsx! {
            HintLine { "Your colors, card roles and top oracle tags, read from every deck you own in any format" }
            HintBullets {
                HintBullet { HintKey { "Create deck" } " begins a deck with that commander" }
                HintBullet { HintKey { "Maybeboard" } " saves it to your commander maybeboard" }
                HintBullet { HintKey { color: "--color-error", "Remove" } " hides it for a week and pulls the next one up" }
                HintBullet { HintKey { "Swipe" } " deals the same ranking: " HintColored { color: "--color-success", "right" } " starts a deck, " HintColored { color: "--color-warning", "up" } " saves, " HintColored { color: "--color-error", "left" } " skips it" }
                HintBullet { HintKey { "Show" } " narrows the list to one color identity" }
            }
            HintLine { "Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking" }
        },
        OneTimeHint::Activity => rsx! {
            HintLine { "How you pick, read from your swipes" }
            HintBullets {
                HintBullet { HintKey { "Activity" } " maps your swipes and searches over the last year, and what the cards dealt to your decks came to" }
                HintBullet { HintKey { "Card roles" } " and " HintKey { "Oracle tags" } " show how often you add, skip, save or remove each one" }
                HintBullet { HintKey { "Most skipped" } " lists cards you skip that other players usually keep" }
            }
            HintLine { "Each section's ? explains it" }
        },
        OneTimeHint::Filter | OneTimeHint::SwipeSelect => rsx! {},
    }
}

/// The `filter` hint, opened the first time a filter section is used on a
/// screen.
#[component]
pub fn FilterHint(open: Signal<bool>) -> Element {
    rsx! {
        HintDialog { open, title: "Filters",
            HintBullets {
                HintBullet { "Picks which cards show: the ones served to " HintColored { color: "--accent-primary", "swipe" } ", or which of the deck's cards list" }
                HintBullet { "Open a section to set name, mana, type, color, " HintColored { color: "--accent-tertiary", "Oracle tags" } " and more. Stack as many as you want" }
                HintBullet { HintKey { "Apply" } " uses it. " HintKey { color: "--color-warning", "Reset" } " then " HintKey { "Apply" } " goes back to the default view" }
                HintBullet { "Each screen keeps its own filter" }
            }
        }
    }
}

/// The `swipe_select` hint. `suggested` is the Playstyle ranking's mode,
/// titled `Suggested commanders`; otherwise `noun` names what a right swipe
/// picks (`commander`, `partner`, `background`, `signature spell` or
/// `oathbreaker`). `has_filter` adds the Filter line in suggested mode.
#[component]
pub fn SwipeSelectHint(
    open: Signal<bool>,
    suggested: bool,
    noun: String,
    #[props(default)] has_filter: bool,
) -> Element {
    let title = if suggested {
        "Suggested commanders"
    } else {
        "Swipe select"
    };
    rsx! {
        HintDialog { open, title,
            HintBullets {
                if suggested {
                    HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " starts a deck with that commander" }
                    HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it for a week" }
                } else {
                    HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " picks that {noun}" }
                    HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it" }
                }
                HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " saves it to your commander maybeboard" }
                HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
            }
            if suggested {
                HintLine { "Best fit first, 100 deep. The eye shows why each one was suggested" }
                if has_filter {
                    HintLine { HintKey { "Filter" } " narrows the stack to one color identity" }
                }
            } else {
                HintLine { "Most-played first. " HintKey { "Filter" } " narrows by color or anything else" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [OneTimeHint; 20] = [
        OneTimeHint::FirstLogin,
        OneTimeHint::Profile,
        OneTimeHint::Config,
        OneTimeHint::UniversesBeyondExceptions,
        OneTimeHint::DeckList,
        OneTimeHint::CreateDeck,
        OneTimeHint::EditDeck,
        OneTimeHint::FirstDeck,
        OneTimeHint::DeckCards,
        OneTimeHint::AddDeckCards,
        OneTimeHint::RemoveDeckCards,
        OneTimeHint::Filter,
        OneTimeHint::Import,
        OneTimeHint::Export,
        OneTimeHint::OracleTagDictionary,
        OneTimeHint::OtagExamples,
        OneTimeHint::CommanderMaybeboard,
        OneTimeHint::SuggestedCommanders,
        OneTimeHint::Activity,
        OneTimeHint::SwipeSelect,
    ];

    #[test]
    fn every_key_is_one_the_server_accepts() {
        for hint in ALL {
            let key = hint.key();
            assert!(
                !key.is_empty()
                    && key.len() <= hints::HINT_KEY_MAX_LEN
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
                "{hint:?} has key {key:?}"
            );
        }
    }

    #[test]
    fn every_hint_has_its_own_key() {
        let mut keys: Vec<&str> = ALL.iter().map(|h| h.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), ALL.len());
    }

    #[test]
    fn only_the_two_with_props_lack_a_fixed_title() {
        for hint in ALL {
            let fixed = fixed_title(hint).is_some();
            let has_props = matches!(hint, OneTimeHint::Filter | OneTimeHint::SwipeSelect);
            assert_eq!(fixed, !has_props, "{hint:?}");
        }
    }
}
