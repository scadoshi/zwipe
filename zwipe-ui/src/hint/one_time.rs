//! The one-time hints: the dialog each [`OneTimeHint`] opens the first
//! time a user reaches its screen, word for word as the app shows it.
//!
//! Whether a user has seen one is the server's to record, under
//! [`OneTimeHint::key`]; this module only holds what the dialogs say.

use zwipe_core::domain::user::hints::OneTimeHint;

use super::{
    Block, HintContent, KeyIcon, Role, bullet, emph, icon_key, icon_key_in, key, key_in, line, text,
};
use crate::deck::form::Slot;

/// Which overlay the Swipe select hint opens over; its copy changes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeSelectMode {
    /// Picking a card for a command-zone slot on Create or Edit, named by
    /// [`crate::deck::form::Slot::noun`]: `commander`, `oathbreaker`,
    /// `partner`, `background` or `signature spell`.
    Picking(&'static str),
    /// Swiping through commanders from the Commander maybeboard, where a pick
    /// starts a new deck with it.
    StartsDeck,
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
        bullet(vec![text(
            "Deck tags and Oracle tags shape which cards we suggest",
        )]),
        bullet(vec![
            text("Tap a field's "),
            key_in("?", Role::Palette6),
            text(" for what it does"),
        ]),
    ]
}

/// Which deck page the first-deck hint describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeckLayout {
    /// The app, and the web below its breakpoint: the dock holds the pencil,
    /// clone and share, then the `Cards` pill.
    Narrow,
    /// The wide web page: one dock holds add and remove, the filter and
    /// undo, then edit, clone and share.
    Wide,
}

/// The first-deck hint for the deck page's `layout`.
#[must_use]
pub fn first_deck(layout: DeckLayout) -> HintContent {
    let mut blocks = if layout == DeckLayout::Wide {
        vec![
            bullet(vec![
                icon_key(KeyIcon::Plus),
                text(" adds cards and "),
                icon_key(KeyIcon::Minus),
                text(" removes them"),
            ]),
            bullet(vec![
                icon_key(KeyIcon::Funnel),
                text(" filters the cards (its "),
                key("\u{d7}"),
                text(" resets the filter), and "),
                icon_key(KeyIcon::Undo),
                text(" undoes"),
            ]),
            bullet(vec![
                icon_key(KeyIcon::Edit),
                text(" changes the name, format, command zone, tags and targets"),
            ]),
            bullet(vec![
                icon_key(KeyIcon::Clone),
                text(" clones the deck and "),
                icon_key(KeyIcon::Share),
                text(" shares a link to it"),
            ]),
        ]
    } else {
        vec![
            bullet(vec![key("Cards"), text(" browses, adds and removes cards")]),
            bullet(vec![
                text("In the dock, "),
                icon_key(KeyIcon::Edit),
                text(" changes the name, format, command zone, tags and targets"),
            ]),
            bullet(vec![
                text("Beside it, "),
                icon_key(KeyIcon::Clone),
                text(" clones the deck and "),
                icon_key(KeyIcon::Share),
                text(" shares a link to it"),
            ]),
        ]
    };
    blocks.extend([
        bullet(vec![text(
            "Import, export and delete are at the bottom of the page",
        )]),
        bullet(vec![
            text("Skipped and removed cards stay out of the add stack. "),
            key("Clear"),
            text(" at the bottom brings them back"),
        ]),
        bullet(vec![text(
            "Stats fill in as the deck grows. Tap a section to expand it",
        )]),
        bullet(vec![text(
            "Warnings flag rule problems, each with a one-tap fix",
        )]),
    ]);
    content("Welcome to your deck", blocks)
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
                bullet(vec![
                    key("Decks"),
                    text(" lists your latest decks. "),
                    key("Create a deck"),
                    text(" starts one, and once you have decks "),
                    key("All decks"),
                    text(" opens the list"),
                ]),
                bullet(vec![
                    key("Colors"),
                    text(" counts your decks' cards by color. Tap "),
                    key("Playstyle"),
                    text(" for suggested commanders"),
                ]),
                bullet(vec![
                    key("Account"),
                    text(", the "),
                    icon_key(KeyIcon::Person),
                    text(
                        " in the menu's footer (the top bar on a wide window), has your email, password and log out",
                    ),
                ]),
                bullet(vec![
                    key("Settings"),
                    text(", the "),
                    icon_key(KeyIcon::Cog),
                    text(" beside it, sets your theme and which cards show"),
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
                bullet(vec![
                    key("Danger zone"),
                    text(" at the bottom logs you out or deletes your account"),
                ]),
            ],
        ),
        OneTimeHint::Config => content(
            "Settings",
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
                    text(" groups the list by format, color or tag, "),
                    key("Show"),
                    text(" narrows it by color or tag"),
                ]),
                bullet(vec![
                    text("The "),
                    icon_key(KeyIcon::Plus),
                    text(" at the bottom right starts a new deck"),
                ]),
            ],
        ),
        OneTimeHint::FirstDeck => first_deck(DeckLayout::Narrow),
        OneTimeHint::DeckCards => content(
            "Browsing your deck",
            vec![
                line(vec![text("Tap a row for details, quantity and printings")]),
                bullet(vec![
                    text("Tap a "),
                    emph("keyword", Role::Palette1),
                    text(" chip for what it does, or a "),
                    emph("role", Role::Palette1),
                    text(" chip for its Oracle tags"),
                ]),
                bullet(vec![
                    key("Group by"),
                    text(" groups by type, mana value, color or role"),
                ]),
                bullet(vec![key("Boards"), text(" picks main, maybe or side")]),
                bullet(vec![
                    key("Show"),
                    text(" toggles tokens, the command zone and row art"),
                ]),
                bullet(vec![
                    text("The bar at the bottom: "),
                    icon_key(KeyIcon::Plus),
                    text(" adds, "),
                    icon_key(KeyIcon::Minus),
                    text(" removes, "),
                    icon_key(KeyIcon::Funnel),
                    text(" filters (its "),
                    key("\u{d7}"),
                    text(" resets the filter), and "),
                    icon_key(KeyIcon::Undo),
                    text(" undoes"),
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
                    text("The chip after "),
                    key("From"),
                    text(" switches between Search and the maybeboard"),
                ]),
                bullet(vec![
                    icon_key(KeyIcon::Bolt),
                    text(
                        " is Synergy: on starts with cards that work with your commander, then cards like your deck. Off shows every legal card",
                    ),
                ]),
                line(vec![text(
                    "The filter's Sort only reorders. It never hides cards",
                )]),
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
                    text(" moves it between mainboard and maybeboard"),
                ]),
                bullet(vec![
                    text("Swipe "),
                    emph("down", Role::Tertiary),
                    text(" or "),
                    icon_key(KeyIcon::Undo),
                    text(" undoes the last swipe, even at the end of the stack"),
                ]),
                line(vec![text(
                    "The board chips at the bottom left pick which board you are trimming",
                )]),
            ],
        ),
        OneTimeHint::Filter => content(
            "Filters",
            vec![
                bullet(vec![
                    text("Picks which cards show: the ones dealt to "),
                    emph("swipe", Role::Primary),
                    text(", or the deck's cards in the list"),
                ]),
                bullet(vec![
                    text("Open a section to set name, mana, type, color, "),
                    emph("Oracle tags", Role::Tertiary),
                    text(" and more. Stack as many as you want"),
                ]),
                bullet(vec![
                    icon_key(KeyIcon::Check),
                    text(" uses it. "),
                    icon_key_in(KeyIcon::Eraser, Role::Warning),
                    text(" then "),
                    icon_key(KeyIcon::Check),
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
                    key("Archidekt"),
                    text(" also sets the deck's format and commander, if the deck has none yet"),
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
                    text("The "),
                    icon_key(KeyIcon::Copy),
                    text(" at the bottom right puts the decklist on your clipboard"),
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
                    text(
                        " deals commanders you haven't saved: left skips, up saves, right starts a deck",
                    ),
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
                    "Your colors, card roles and top Oracle tags from every deck you own in any format",
                )]),
                line(vec![text(
                    "Below them, your top 10 commanders ranked against all your decks, best fit first",
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
            ],
        ),
        OneTimeHint::Activity => content(
            "Activity",
            vec![
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
                    text(" show each one's "),
                    key("Added"),
                    text(", "),
                    key("Skipped"),
                    text(", "),
                    key("Maybeboard"),
                    text(" and "),
                    key("Removed"),
                    text(" shares"),
                ]),
                bullet(vec![
                    key("Most skipped"),
                    text(" lists cards you skip that other players usually keep"),
                ]),
                line(vec![
                    text("Each section's "),
                    key_in("?", Role::Palette6),
                    text(" explains it"),
                ]),
            ],
        ),
        OneTimeHint::CreateDeck | OneTimeHint::EditDeck => {
            content("Building a deck", building_a_deck())
        }
        OneTimeHint::SwipeSelect => return None,
    })
}

/// The Swipe select bullet for an up swipe, which saves to the commander maybeboard.
fn up_saves() -> Block {
    bullet(vec![
        text("Swipe "),
        emph("up", Role::Warning),
        text(" saves it to your commander maybeboard"),
    ])
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
        SwipeSelectMode::StartsDeck => vec![
            bullet(vec![
                text("Swipe "),
                emph("left", Role::Error),
                text(" skips it"),
            ]),
            up_saves(),
            bullet(vec![
                text("Swipe "),
                emph("right", Role::Success),
                text(" starts a new deck with that commander"),
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
    // Up saves only where the pile holds commanders: not for a background
    // or a signature spell. From the maybeboard it already sits between
    // left and right.
    let no_commanders = [Slot::Background, Slot::SignatureSpell].map(|slot| slot.noun(None));
    let up_listed = match mode {
        SwipeSelectMode::Picking(noun) => no_commanders.contains(&noun),
        SwipeSelectMode::StartsDeck => true,
        SwipeSelectMode::Suggested { .. } => false,
    };
    if !up_listed {
        blocks.push(up_saves());
    }
    blocks.push(bullet(vec![
        text("Swipe "),
        emph("down", Role::Tertiary),
        text(" undoes the last swipe"),
    ]));
    let title = match mode {
        SwipeSelectMode::Suggested { has_filter } => {
            blocks.push(line(vec![
                text("Best fit first, 100 deep. "),
                icon_key(KeyIcon::Eye),
                text(" shows why each one was suggested"),
            ]));
            if has_filter {
                blocks.push(line(vec![
                    icon_key(KeyIcon::Funnel),
                    text(" narrows the stack to one color identity"),
                ]));
            }
            "Suggested commanders"
        }
        SwipeSelectMode::Picking(_) | SwipeSelectMode::StartsDeck => {
            blocks.push(line(vec![
                text("Most-played first. "),
                icon_key(KeyIcon::Funnel),
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
            bullet(vec![
                key("Decks"),
                text(" lists your latest decks. "),
                key("Create a deck"),
                text(" starts one, and once you have decks "),
                key("All decks"),
                text(" opens the list"),
            ])
        );
    }

    #[test]
    fn from_the_maybeboard_left_skips_up_saves_and_right_starts_a_deck() {
        let hint = swipe_select(SwipeSelectMode::StartsDeck);
        assert_eq!(hint.title, "Swipe select");
        let lines: Vec<String> = hint.blocks.iter().take(4).map(Block::plain).collect();
        assert_eq!(
            lines,
            [
                "Swipe left skips it",
                "Swipe up saves it to your commander maybeboard",
                "Swipe right starts a new deck with that commander",
                "Swipe down undoes the last swipe",
            ],
        );
    }

    #[test]
    fn the_deck_hint_names_the_clear_button_as_drawn() {
        assert!(
            first_deck(DeckLayout::Narrow)
                .blocks
                .iter()
                .any(|b| b.spans().contains(&key("Clear")))
        );
    }

    #[test]
    fn up_saves_only_where_the_pile_holds_commanders() {
        let says_up = |mode| {
            swipe_select(mode)
                .blocks
                .iter()
                .any(|b| b.plain().starts_with("Swipe up"))
        };
        assert!(says_up(SwipeSelectMode::Picking(Slot::Partner.noun(None))));
        assert!(says_up(SwipeSelectMode::Suggested { has_filter: false }));
        assert!(!says_up(SwipeSelectMode::Picking(
            Slot::Background.noun(None)
        )));
        assert!(!says_up(SwipeSelectMode::Picking(
            Slot::SignatureSpell.noun(None)
        )));
    }

    #[test]
    fn the_hints_name_the_shipped_controls() {
        let welcome = content_for(OneTimeHint::FirstLogin).unwrap();
        assert_eq!(
            welcome.blocks[1].plain(),
            "Colors counts your decks' cards by color. Tap Playstyle for suggested commanders"
        );
        let decks = content_for(OneTimeHint::DeckList).unwrap();
        assert_eq!(
            decks.blocks[2].plain(),
            "The plus at the bottom right starts a new deck"
        );
        assert_eq!(
            decks.blocks[2].spans()[1],
            icon_key(KeyIcon::Plus),
            "the add button is an icon, so its key is the icon alone"
        );
        let deck = content_for(OneTimeHint::FirstDeck).unwrap();
        assert_eq!(
            deck.blocks[1].plain(),
            "In the dock, pencil changes the name, format, command zone, tags and targets"
        );
        assert_eq!(
            deck.blocks[2].plain(),
            "Beside it, clone clones the deck and share shares a link to it"
        );
        let export = content_for(OneTimeHint::Export).unwrap();
        assert_eq!(
            export.blocks[1].plain(),
            "The copy at the bottom right puts the decklist on your clipboard"
        );
        let account = content_for(OneTimeHint::Profile).unwrap();
        assert_eq!(
            account.blocks[1].plain(),
            "Danger zone at the bottom logs you out or deletes your account"
        );
    }

    #[test]
    fn the_wide_deck_hint_shows_the_docks_icons_and_no_cards_pill() {
        let wide = first_deck(DeckLayout::Wide);
        assert_eq!(
            wide.blocks[0].plain(),
            "plus adds cards and minus removes them"
        );
        assert_eq!(wide.blocks[0].spans()[0], icon_key(KeyIcon::Plus));
        assert_eq!(wide.blocks[0].spans()[2], icon_key(KeyIcon::Minus));
        assert!(
            wide.blocks
                .iter()
                .all(|b| !b.spans().contains(&key("Cards"))),
            "the wide page has no Cards pill"
        );
        let narrow = first_deck(DeckLayout::Narrow);
        assert_eq!(narrow.blocks[0].spans()[0], key("Cards"));
        assert_eq!(content_for(OneTimeHint::FirstDeck), Some(narrow));
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
            suggested.blocks[5].plain(),
            "funnel narrows the stack to one color identity"
        );
        assert_eq!(suggested.blocks[5].spans()[0], icon_key(KeyIcon::Funnel));
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
