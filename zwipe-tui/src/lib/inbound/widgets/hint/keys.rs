//! The app's one-time hints by their key in `zwipe_core::domain::user::hints`: the dialog each key opens, word for word, in the markup of [`super::markup`].

use zwipe_core::domain::user::hints::{
    HINT_ACTIVITY, HINT_ADD_DECK_CARDS, HINT_COMMANDER_MAYBEBOARD, HINT_CONFIG, HINT_CREATE_DECK,
    HINT_DECK_CARDS, HINT_DECK_LIST, HINT_EDIT_DECK, HINT_EXPORT, HINT_FILTER, HINT_FIRST_DECK,
    HINT_FIRST_LOGIN, HINT_IMPORT, HINT_ORACLE_TAG_DICTIONARY, HINT_OTAG_EXAMPLES, HINT_PROFILE,
    HINT_REMOVE_DECK_CARDS, HINT_SUGGESTED_COMMANDERS, HINT_SWIPE_SELECT,
    HINT_UNIVERSES_BEYOND_EXCEPTIONS,
};

use super::HintContent;

/// Every key with fixed copy. `swipe_select` changes with its overlay, so it is built by [`swipe_select`] instead.
#[cfg(test)]
pub(crate) const KEYS: [&str; 19] = [
    HINT_FIRST_LOGIN,
    HINT_PROFILE,
    HINT_CONFIG,
    HINT_UNIVERSES_BEYOND_EXCEPTIONS,
    HINT_DECK_LIST,
    HINT_CREATE_DECK,
    HINT_EDIT_DECK,
    HINT_FIRST_DECK,
    HINT_DECK_CARDS,
    HINT_ADD_DECK_CARDS,
    HINT_REMOVE_DECK_CARDS,
    HINT_FILTER,
    HINT_IMPORT,
    HINT_EXPORT,
    HINT_ORACLE_TAG_DICTIONARY,
    HINT_OTAG_EXAMPLES,
    HINT_COMMANDER_MAYBEBOARD,
    HINT_SUGGESTED_COMMANDERS,
    HINT_ACTIVITY,
];

/// The building-a-deck hint, shared by Create and Edit.
const BUILDING_A_DECK: &[&str] = &[
    "• {Profile|primary} is what the deck is, {Budget|secondary} what it may cost, {Tags|tertiary} what it does",
    "• Tags shape which cards we suggest",
    "• Tap a field's [?|p6] for what it does",
];

fn content(title: &str, blocks: &[&str]) -> HintContent {
    HintContent {
        title: title.to_owned(),
        blocks: blocks.iter().map(|&b| b.to_owned()).collect(),
    }
}

/// The dialog the one-time hint `key` opens, or `None` for a key with no fixed copy.
pub(crate) fn for_key(key: &str) -> Option<HintContent> {
    Some(match key {
        HINT_FIRST_LOGIN => content(
            "Welcome to Zwipe",
            &[
                "• [Decks] holds your decks",
                "• [Color map] grows from your decks' colors. Tap Playstyle for suggested commanders",
                "• [Account] has your email, password and log out",
                "• [Config] sets your theme and which cards show",
                "• {Verify your email|warning} in Account for full deck and card limits",
            ],
        ),
        HINT_PROFILE => content(
            "Your account",
            &[
                "• Tap a row's [?|p6] for what it does",
                "• [More] deletes your account",
            ],
        ),
        HINT_CONFIG => content(
            "Config",
            &[
                "• Set how Zwipe looks and which cards it shows",
                "• Tap a row's [?|p6] for what it does",
            ],
        ),
        HINT_UNIVERSES_BEYOND_EXCEPTIONS => content(
            "Franchise exceptions",
            &["Picked franchises still show while Universes Beyond is hidden"],
        ),
        HINT_DECK_LIST => content(
            "Your decks",
            &[
                "• Tap a deck to open it",
                "• [Group by] folds the list into sections, [Show] narrows it by color or tag",
                "• [Create] starts a new deck",
            ],
        ),
        HINT_CREATE_DECK | HINT_EDIT_DECK => content("Building a deck", BUILDING_A_DECK),
        HINT_FIRST_DECK => content(
            "Welcome to your deck",
            &[
                "• [Cards] browses, adds and removes cards",
                "• [Edit] changes the name, format, command zone, tags and targets",
                "• [More] imports, exports, clones, shares or deletes the deck",
                "• Skipped and removed cards stay out of the add stack. [More] then Clear skips brings them back",
                "• Stats fill in as the deck grows. Tap a section to expand it",
                "• Warnings flag rule problems, each with a one-tap fix",
            ],
        ),
        HINT_DECK_CARDS => content(
            "Browsing your deck",
            &[
                "Tap a row for details, quantity and printings",
                "• Tap a {keyword|p1} chip for what it does, or a {role|p1} chip for its oracle tags",
                "• [Group by] sorts by type, mana value, color or role",
                "• [Boards] picks main, maybe or side",
                "• [Show] toggles tokens, the command zone and row art",
                "• [Star|warning] marks up to three MVPs. Suggestions lean toward cards that share their roles",
            ],
        ),
        HINT_ADD_DECK_CARDS => content(
            "Swipe to build",
            &[
                "• Swipe {right|success} adds the card",
                "• Swipe {left|error} skips it for a week",
                "• Swipe {up|warning} sends it to the maybeboard",
                "• Swipe {down|tertiary} undoes the last swipe",
                "• [Synergy] on starts with cards that work with your commander, then cards like your deck. Off shows every legal card",
                "Sort only reorders. It never hides cards",
            ],
        ),
        HINT_REMOVE_DECK_CARDS => content(
            "Swipe to trim",
            &[
                "• Swipe {right|success} removes the card",
                "• Swipe {left|error} keeps it",
                "• Swipe {up|warning} moves it between main and maybeboard",
                "• Swipe {down|tertiary} undoes the last swipe",
                "The board chips up top pick which board you are trimming",
            ],
        ),
        HINT_FILTER => content(
            "Filters",
            &[
                "• Picks which cards show: the ones served to {swipe|primary}, or which of the deck's cards list",
                "• Open a section to set name, mana, type, color, {Oracle tags|tertiary} and more. Stack as many as you want",
                "• [Apply] uses it. [Reset|warning] then [Apply] goes back to the default view",
                "• Each screen keeps its own filter",
            ],
        ),
        HINT_IMPORT => content(
            "Importing cards",
            &[
                "• Paste a decklist, or switch [From] to an Archidekt deck link",
                "• [Add] keeps your current cards. [Replace|warning] swaps the board's contents",
                "• [Board] picks where the cards go",
                "• Importing clears the deck's undo history",
            ],
        ),
        HINT_EXPORT => content(
            "Exporting your deck",
            &[
                "• [Export] picks which boards to include",
                "• [Copy] puts the decklist on your clipboard",
            ],
        ),
        HINT_ORACLE_TAG_DICTIONARY => content(
            "Oracle tags",
            &[
                "• Tap a [letter|p1] to browse, or {search|secondary} by name or description",
                "• [Examples] shows real cards with that tag",
                "• [Use] adds the tag where you came from",
                "• Some tags have no description yet",
            ],
        ),
        HINT_OTAG_EXAMPLES => content(
            "Example cards",
            &[
                "• Swipe {left|success} for the next card",
                "• Swipe {down|secondary} to go back one",
            ],
        ),
        HINT_COMMANDER_MAYBEBOARD => content(
            "Commander maybeboard",
            &[
                "Commanders you save land here",
                "• Swipe {up|warning} while picking a commander saves it here",
                "• [Swipe] deals commanders: right starts a deck, up saves one",
                "• [Quick add] searches by name",
                "• [Create deck] starts a deck with that commander",
                "• [Remove|error] takes it off the list",
            ],
        ),
        HINT_SUGGESTED_COMMANDERS => content(
            "Playstyle",
            &[
                "Your colors, card roles and top oracle tags, read from every deck you own in any format",
                "• [Create deck] begins a deck with that commander",
                "• [Maybeboard] saves it to your commander maybeboard",
                "• [Remove|error] hides it for a week and pulls the next one up",
                "• [Swipe] deals the same ranking: {right|success} starts a deck, {up|warning} saves, {left|error} skips it",
                "• [Show] narrows the list to one color identity",
                "Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking",
            ],
        ),
        HINT_ACTIVITY => content(
            "Activity",
            &[
                "How you pick, read from your swipes",
                "• [Activity] maps your swipes and searches over the last year, and what the cards dealt to your decks came to",
                "• [Card roles] and [Oracle tags] show how often you add, skip, save or remove each one",
                "• [Most skipped] lists cards you skip that other players usually keep",
                "Each section's ? explains it",
            ],
        ),
        _ => return None,
    })
}

/// The `swipe_select` hint, which changes with its overlay: `suggested` for the suggested-commanders swipe, else picking a `noun` (`oathbreaker`, `commander`, `partner`, `background` or `signature spell`); `has_filter` adds the suggested mode's filter line.
pub(crate) fn swipe_select(suggested: bool, noun: &str, has_filter: bool) -> HintContent {
    let mut blocks: Vec<String> = if suggested {
        vec![
            "• Swipe {right|success} starts a deck with that commander".to_owned(),
            "• Swipe {left|error} skips it for a week".to_owned(),
        ]
    } else {
        vec![
            format!("• Swipe {{right|success}} picks that {noun}"),
            "• Swipe {left|error} skips it".to_owned(),
        ]
    };
    blocks.extend([
        "• Swipe {up|warning} saves it to your commander maybeboard".to_owned(),
        "• Swipe {down|tertiary} undoes the last swipe".to_owned(),
    ]);
    if suggested {
        blocks
            .push("Best fit first, 100 deep. The eye shows why each one was suggested".to_owned());
        if has_filter {
            blocks.push("[Filter] narrows the stack to one color identity".to_owned());
        }
    } else {
        blocks.push("Most-played first. [Filter] narrows by color or anything else".to_owned());
    }
    HintContent {
        title: if suggested {
            "Suggested commanders"
        } else {
            "Swipe select"
        }
        .to_owned(),
        blocks,
    }
}

/// The key the `swipe_select` hint is stored under, for [`super::once_with`].
pub(crate) const SWIPE_SELECT: &str = HINT_SWIPE_SELECT;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::widgets::hint::markup::plain;

    #[test]
    fn every_key_has_its_dialog_and_unknown_keys_none() {
        for key in KEYS {
            let content = for_key(key).unwrap();
            assert!(
                !content.title.is_empty() && !content.blocks.is_empty(),
                "{key}"
            );
        }
        assert!(for_key("nope").is_none());
        assert!(for_key(SWIPE_SELECT).is_none(), "built per overlay");
    }

    #[test]
    fn create_and_edit_share_their_copy() {
        assert_eq!(for_key(HINT_CREATE_DECK), for_key(HINT_EDIT_DECK));
        let welcome = for_key(HINT_FIRST_LOGIN).unwrap();
        assert_eq!(welcome.title, "Welcome to Zwipe");
        assert_eq!(plain(&welcome.blocks[0]), "•  Decks  holds your decks");
    }

    #[test]
    fn swipe_select_names_its_noun_or_the_suggested_mode() {
        let picking = swipe_select(false, "partner", false);
        assert_eq!(picking.title, "Swipe select");
        assert_eq!(
            plain(&picking.blocks[0]),
            "• Swipe right picks that partner"
        );
        let suggested = swipe_select(true, "commander", true);
        assert_eq!(suggested.title, "Suggested commanders");
        assert_eq!(suggested.blocks.len(), 6);
        assert_eq!(swipe_select(true, "commander", false).blocks.len(), 5);
    }
}
