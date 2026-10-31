//! The one-time hints: the dialog each [`OneTimeHint`] opens the first
//! time a user reaches its screen, word for word as the app shows it.
//!
//! Whether a user has seen one is the server's to record, under
//! [`OneTimeHint::key`]; this module only holds what the dialogs say.

use zwipe_core::domain::user::hints::OneTimeHint;

use super::{Block, HintContent, Role, bullet, emph, key, key_in, line, text};

/// Which overlay the Swipe select hint opens over; its copy changes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeSelectMode {
    /// Picking a card for a command-zone slot on Create or Edit, named by
    /// [`crate::deck::form::Slot::noun`]: `commander`, `oathbreaker`,
    /// `partner`, `background` or `signature spell`.
    Picking(&'static str),
    /// Swiping through Playstyle's suggested commanders.
    Suggested {
        /// Whether the overlay has a filter, which adds a line about it.
        has_filter: bool,
    },
}

/// The building-a-deck hint, which Create and Edit share.
fn building_a_deck() -> Vec<Block> {
    vec![
        bullet(vec![
            emph("Profile", Role::Primary),
            text(" is what the deck is, "),
            emph("Budget", Role::Secondary),
            text(" what it may cost, "),
            emph("Tags", Role::Tertiary),
            text(" what it does"),
        ]),
        bullet(vec![text("Tags shape which cards we suggest")]),
        bullet(vec![
            text("Tap a field's "),
            key_in("?", Role::Palette6),
            text(" for what it does"),
        ]),
    ]
}

/// A dialog's content from its title and body.
fn content(title: &str, blocks: Vec<Block>) -> HintContent {
    HintContent {
        title: title.to_owned(),
        blocks,
    }
}

/// The dialog `hint` opens, or `None` for [`OneTimeHint::SwipeSelect`],
/// whose copy depends on its overlay: see [`swipe_select`].
#[must_use]
#[expect(clippy::too_many_lines, reason = "a lookup table, one arm per hint")]
pub fn content_for(hint: OneTimeHint) -> Option<HintContent> {
    Some(match hint {
        OneTimeHint::FirstLogin => content(
            "Welcome to Zwipe",
            vec![
                bullet(vec![key("Decks"), text(" holds your decks")]),
                bullet(vec![
                    key("Color map"),
                    text(" grows from your decks' colors. Tap Playstyle for suggested commanders"),
                ]),
                bullet(vec![
                    key("Account"),
                    text(" has your email, password and log out"),
                ]),
                bullet(vec![
                    key("Config"),
                    text(" sets your theme and which cards show"),
                ]),
                bullet(vec![
                    emph("Verify your email", Role::Warning),
                    text(" in Account for full deck and card limits"),
                ]),
            ],
        ),
        OneTimeHint::Profile => content(
            "Your account",
            vec![
                bullet(vec![
                    text("Tap a row's "),
                    key_in("?", Role::Palette6),
                    text(" for what it does"),
                ]),
                bullet(vec![key("More"), text(" deletes your account")]),
            ],
        ),
        OneTimeHint::Config => content(
            "Config",
            vec![
                bullet(vec![text("Set how Zwipe looks and which cards it shows")]),
                bullet(vec![
                    text("Tap a row's "),
                    key_in("?", Role::Palette6),
                    text(" for what it does"),
                ]),
            ],
        ),
        OneTimeHint::UniversesBeyondExceptions => content(
            "Franchise exceptions",
            vec![line(vec![text(
                "Picked franchises still show while Universes Beyond is hidden",
            )])],
        ),
        OneTimeHint::DeckList => content(
            "Your decks",
            vec![
                bullet(vec![text("Tap a deck to open it")]),
                bullet(vec![
                    key("Group by"),
                    text(" folds the list into sections, "),
                    key("Show"),
                    text(" narrows it by color or tag"),
                ]),
                bullet(vec![key("Create"), text(" starts a new deck")]),
            ],
        ),
        OneTimeHint::FirstDeck => content(
            "Welcome to your deck",
            vec![
                bullet(vec![key("Cards"), text(" browses, adds and removes cards")]),
                bullet(vec![
                    key("Edit"),
                    text(" changes the name, format, command zone, tags and targets"),
                ]),
                bullet(vec![
                    key("More"),
                    text(" imports, exports, clones, shares or deletes the deck"),
                ]),
                bullet(vec![
                    text("Skipped and removed cards stay out of the add stack. "),
                    key("More"),
                    text(" then Clear skips brings them back"),
                ]),
                bullet(vec![text(
                    "Stats fill in as the deck grows. Tap a section to expand it",
                )]),
                bullet(vec![text(
                    "Warnings flag rule problems, each with a one-tap fix",
                )]),
            ],
        ),
        OneTimeHint::DeckCards => content(
            "Browsing your deck",
            vec![
                line(vec![text("Tap a row for details, quantity and printings")]),
                bullet(vec![
                    text("Tap a "),
                    emph("keyword", Role::Palette1),
                    text(" chip for what it does, or a "),
                    emph("role", Role::Palette1),
                    text(" chip for its oracle tags"),
                ]),
                bullet(vec![
                    key("Group by"),
                    text(" sorts by type, mana value, color or role"),
                ]),
                bullet(vec![key("Boards"), text(" picks main, maybe or side")]),
                bullet(vec![
                    key("Show"),
                    text(" toggles tokens, the command zone and row art"),
                ]),
                bullet(vec![
                    key_in("Star", Role::Warning),
                    text(
                        " marks up to three MVPs. Suggestions lean toward cards that share their roles",
                    ),
                ]),
            ],
        ),
        OneTimeHint::AddDeckCards => content(
            "Swipe to build",
            vec![
                bullet(vec![
                    text("Swipe "),
                    emph("right", Role::Success),
                    text(" adds the card"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("left", Role::Error),
                    text(" skips it for a week"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("up", Role::Warning),
                    text(" sends it to the maybeboard"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("down", Role::Tertiary),
                    text(" undoes the last swipe"),
                ]),
                bullet(vec![
                    key("Synergy"),
                    text(
                        " on starts with cards that work with your commander, then cards like your deck. Off shows every legal card",
                    ),
                ]),
                line(vec![text("Sort only reorders. It never hides cards")]),
            ],
        ),
        OneTimeHint::RemoveDeckCards => content(
            "Swipe to trim",
            vec![
                bullet(vec![
                    text("Swipe "),
                    emph("right", Role::Success),
                    text(" removes the card"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("left", Role::Error),
                    text(" keeps it"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("up", Role::Warning),
                    text(" moves it between main and maybeboard"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("down", Role::Tertiary),
                    text(" undoes the last swipe"),
                ]),
                line(vec![text(
                    "The board chips up top pick which board you are trimming",
                )]),
            ],
        ),
        OneTimeHint::Filter => content(
            "Filters",
            vec![
                bullet(vec![
                    text("Picks which cards show: the ones served to "),
                    emph("swipe", Role::Primary),
                    text(", or which of the deck's cards list"),
                ]),
                bullet(vec![
                    text("Open a section to set name, mana, type, color, "),
                    emph("Oracle tags", Role::Tertiary),
                    text(" and more. Stack as many as you want"),
                ]),
                bullet(vec![
                    key("Apply"),
                    text(" uses it. "),
                    key_in("Reset", Role::Warning),
                    text(" then "),
                    key("Apply"),
                    text(" goes back to the default view"),
                ]),
                bullet(vec![text("Each screen keeps its own filter")]),
            ],
        ),
        OneTimeHint::Import => content(
            "Importing cards",
            vec![
                bullet(vec![
                    text("Paste a decklist, or switch "),
                    key("From"),
                    text(" to an Archidekt deck link"),
                ]),
                bullet(vec![
                    key("Add"),
                    text(" keeps your current cards. "),
                    key_in("Replace", Role::Warning),
                    text(" swaps the board's contents"),
                ]),
                bullet(vec![key("Board"), text(" picks where the cards go")]),
                bullet(vec![text("Importing clears the deck's undo history")]),
            ],
        ),
        OneTimeHint::Export => content(
            "Exporting your deck",
            vec![
                bullet(vec![key("Export"), text(" picks which boards to include")]),
                bullet(vec![
                    key("Copy"),
                    text(" puts the decklist on your clipboard"),
                ]),
            ],
        ),
        OneTimeHint::OracleTagDictionary => content(
            "Oracle tags",
            vec![
                bullet(vec![
                    text("Tap a "),
                    key_in("letter", Role::Palette1),
                    text(" to browse, or "),
                    emph("search", Role::Secondary),
                    text(" by name or description"),
                ]),
                bullet(vec![
                    key("Examples"),
                    text(" shows real cards with that tag"),
                ]),
                bullet(vec![key("Use"), text(" adds the tag where you came from")]),
                bullet(vec![text("Some tags have no description yet")]),
            ],
        ),
        OneTimeHint::OtagExamples => content(
            "Example cards",
            vec![
                bullet(vec![
                    text("Swipe "),
                    emph("left", Role::Success),
                    text(" for the next card"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("down", Role::Secondary),
                    text(" to go back one"),
                ]),
            ],
        ),
        OneTimeHint::CommanderMaybeboard => content(
            "Commander maybeboard",
            vec![
                line(vec![text("Commanders you save land here")]),
                bullet(vec![
                    text("Swipe "),
                    emph("up", Role::Warning),
                    text(" while picking a commander saves it here"),
                ]),
                bullet(vec![
                    key("Swipe"),
                    text(" deals commanders: right starts a deck, up saves one"),
                ]),
                bullet(vec![key("Quick add"), text(" searches by name")]),
                bullet(vec![
                    key("Create deck"),
                    text(" starts a deck with that commander"),
                ]),
                bullet(vec![
                    key_in("Remove", Role::Error),
                    text(" takes it off the list"),
                ]),
            ],
        ),
        OneTimeHint::SuggestedCommanders => content(
            "Playstyle",
            vec![
                line(vec![text(
                    "Your colors, card roles and top oracle tags, read from every deck you own in any format",
                )]),
                bullet(vec![
                    key("Create deck"),
                    text(" begins a deck with that commander"),
                ]),
                bullet(vec![
                    key("Maybeboard"),
                    text(" saves it to your commander maybeboard"),
                ]),
                bullet(vec![
                    key_in("Remove", Role::Error),
                    text(" hides it for a week and pulls the next one up"),
                ]),
                bullet(vec![
                    key("Swipe"),
                    text(" deals the same ranking: "),
                    emph("right", Role::Success),
                    text(" starts a deck, "),
                    emph("up", Role::Warning),
                    text(" saves, "),
                    emph("left", Role::Error),
                    text(" skips it"),
                ]),
                bullet(vec![
                    key("Show"),
                    text(" narrows the list to one color identity"),
                ]),
                line(vec![text(
                    "Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking",
                )]),
            ],
        ),
        OneTimeHint::Activity => content(
            "Activity",
            vec![
                line(vec![text("How you pick, read from your swipes")]),
                bullet(vec![
                    key("Activity"),
                    text(
                        " maps your swipes and searches over the last year, and what the cards dealt to your decks came to",
                    ),
                ]),
                bullet(vec![
                    key("Card roles"),
                    text(" and "),
                    key("Oracle tags"),
                    text(" show how often you add, skip, save or remove each one"),
                ]),
                bullet(vec![
                    key("Most skipped"),
                    text(" lists cards you skip that other players usually keep"),
                ]),
                line(vec![text("Each section's ? explains it")]),
            ],
        ),
        OneTimeHint::CreateDeck | OneTimeHint::EditDeck => {
            content("Building a deck", building_a_deck())
        }
        OneTimeHint::SwipeSelect => return None,
    })
}

/// The Swipe select hint, which changes with its overlay.
#[must_use]
pub fn swipe_select(mode: SwipeSelectMode) -> HintContent {
    let mut blocks = match mode {
        SwipeSelectMode::Suggested { .. } => vec![
            bullet(vec![
                text("Swipe "),
                emph("right", Role::Success),
                text(" starts a deck with that commander"),
            ]),
            bullet(vec![
                text("Swipe "),
                emph("left", Role::Error),
                text(" skips it for a week"),
            ]),
        ],
        SwipeSelectMode::Picking(noun) => vec![
            bullet(vec![
                text("Swipe "),
                emph("right", Role::Success),
                text(&format!(" picks that {noun}")),
            ]),
            bullet(vec![
                text("Swipe "),
                emph("left", Role::Error),
                text(" skips it"),
            ]),
        ],
    };
    blocks.extend([
        bullet(vec![
            text("Swipe "),
            emph("up", Role::Warning),
            text(" saves it to your commander maybeboard"),
        ]),
        bullet(vec![
            text("Swipe "),
            emph("down", Role::Tertiary),
            text(" undoes the last swipe"),
        ]),
    ]);
    let title = match mode {
        SwipeSelectMode::Suggested { has_filter } => {
            blocks.push(line(vec![text(
                "Best fit first, 100 deep. The eye shows why each one was suggested",
            )]));
            if has_filter {
                blocks.push(line(vec![
                    key("Filter"),
                    text(" narrows the stack to one color identity"),
                ]));
            }
            "Suggested commanders"
        }
        SwipeSelectMode::Picking(_) => {
            blocks.push(line(vec![
                text("Most-played first. "),
                key("Filter"),
                text(" narrows by color or anything else"),
            ]));
            "Swipe select"
        }
    };
    content(title, blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::form::Slot;

    #[test]
    fn every_hint_but_swipe_select_has_its_dialog() {
        for hint in OneTimeHint::ALL {
            let content = content_for(hint);
            if hint == OneTimeHint::SwipeSelect {
                assert!(content.is_none(), "built per overlay");
                continue;
            }
            let content = content.unwrap();
            assert!(
                !content.title.is_empty() && !content.blocks.is_empty(),
                "{hint:?}"
            );
        }
    }

    #[test]
    fn create_and_edit_share_their_copy() {
        assert_eq!(
            content_for(OneTimeHint::CreateDeck),
            content_for(OneTimeHint::EditDeck)
        );
        let welcome = content_for(OneTimeHint::FirstLogin).unwrap();
        assert_eq!(welcome.title, "Welcome to Zwipe");
        assert_eq!(
            welcome.blocks[0],
            bullet(vec![key("Decks"), text(" holds your decks")])
        );
    }

    #[test]
    fn swipe_select_names_its_slot_or_the_suggested_mode() {
        let picking = swipe_select(SwipeSelectMode::Picking(Slot::Partner.noun(None)));
        assert_eq!(picking.title, "Swipe select");
        assert_eq!(picking.blocks[0].plain(), "Swipe right picks that partner");
        let suggested = swipe_select(SwipeSelectMode::Suggested { has_filter: true });
        assert_eq!(suggested.title, "Suggested commanders");
        assert_eq!(suggested.blocks.len(), 6);
        assert_eq!(
            swipe_select(SwipeSelectMode::Suggested { has_filter: false })
                .blocks
                .len(),
            5
        );
        assert_eq!(
            swipe_select(SwipeSelectMode::Picking(Slot::SignatureSpell.noun(None))).blocks[0]
                .plain(),
            "Swipe right picks that signature spell"
        );
    }
}
