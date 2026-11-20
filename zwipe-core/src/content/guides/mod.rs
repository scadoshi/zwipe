//! The guides zwipe.net and the web app both draw: each one's text, figures and screenshots as
//! data, so the two sites render one copy. Keep articles accurate to the app: labels here mirror
//! the real UI text, `backticks` mark a keyword and `[[name]]` draws an icon button as its icon
//! (a name zwipe-ui's hint `KeyIcon` knows), so no line describes an icon in words. Never
//! describe where synergy data comes from, only what it does for the user.
//!
//! The screenshots are files served once, by zwipe.net ([`image_url`]); [`image_size`] gives each
//! one's size so a page holds its place while it loads.

use crate::content::Surface;

/// A width and a height in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixels {
    /// Across.
    pub width: u32,
    /// Down.
    pub height: u32,
}

/// The size of a phone capture, which nearly every guide screenshot is.
const PHONE_CAPTURE: Pixels = Pixels {
    width: 860,
    height: 1862,
};

/// A guide screenshot's size, for its frame to hold while it loads. A test where the files live
/// checks every one against it.
#[must_use]
pub fn image_size(file: &str) -> Pixels {
    match file {
        "share-your-deck/02-shared-page-app.webp" => Pixels {
            width: 1236,
            height: 1213,
        },
        "keyboard-shortcuts/01-shortcuts-sheet.webp" => Pixels {
            width: 1440,
            height: 900,
        },
        _ => PHONE_CAPTURE,
    }
}

/// Where a guide screenshot is served: once, by zwipe.net, under `/guide-images/`, so zwipe.net
/// and the web app load the same file. A changed screenshot takes a new file name, since the
/// address carries no hash.
#[must_use]
pub fn image_url(file: &str) -> String {
    format!("{}/guide-images/{file}", crate::domain::site::WEB_BASE)
}

/// The files `guide`'s screenshots name, in order, every surface's included: zwipe.net serves
/// the images the web app's guides show too.
#[must_use]
pub fn screenshots(guide: &Guide) -> Vec<&'static str> {
    guide
        .all_blocks()
        .into_iter()
        .filter_map(|block| match block {
            Block::Image { file, .. } => Some(*file),
            Block::Only(..)
            | Block::Lead(_)
            | Block::H2(_)
            | Block::P(_)
            | Block::Steps(_)
            | Block::Bullets(_)
            | Block::Swipe(_)
            | Block::Note(_)
            | Block::Diagram(_) => None,
        })
        .collect()
}

/// The guide whose slug is `slug`.
#[must_use]
pub fn find(slug: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|guide| guide.slug == slug)
}

/// `blocks` as `surface` reads them, in order: a [`Block::Only`] for `surface` opens into its
/// blocks and one for the other surface drops out.
fn flatten_on(blocks: &'static [Block], surface: Option<Surface>, out: &mut Vec<&'static Block>) {
    for block in blocks {
        match block {
            Block::Only(on, inner) => {
                if surface.is_none_or(|surface| surface == *on) {
                    flatten_on(inner, surface, out);
                }
            }
            Block::Lead(_)
            | Block::H2(_)
            | Block::P(_)
            | Block::Steps(_)
            | Block::Bullets(_)
            | Block::Swipe(_)
            | Block::Note(_)
            | Block::Diagram(_)
            | Block::Image { .. } => out.push(block),
        }
    }
}

impl Guide {
    /// The guide's blocks as `surface` shows them: shared ones, and its own [`Block::Only`]s in
    /// their places. Never a [`Block::Only`] itself.
    #[must_use]
    pub fn blocks_on(&self, surface: Surface) -> Vec<&'static Block> {
        let mut out = Vec::new();
        flatten_on(self.blocks, Some(surface), &mut out);
        out
    }

    /// Every block the guide has on any surface, each surface's own included: what zwipe.net
    /// serves the images for and what checks read. Never a [`Block::Only`] itself.
    #[must_use]
    pub fn all_blocks(&self) -> Vec<&'static Block> {
        let mut out = Vec::new();
        flatten_on(self.blocks, None, &mut out);
        out
    }
}

/// A block of article content, drawn in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    /// Emphasized intro paragraph.
    Lead(&'static str),
    /// Section heading.
    H2(&'static str),
    /// Body paragraph.
    P(&'static str),
    /// Numbered steps.
    Steps(&'static [&'static str]),
    /// Bulleted list.
    Bullets(&'static [&'static str]),
    /// Swipe legend: `(direction, meaning)`. Direction is colored to match the app.
    Swipe(&'static [(&'static str, &'static str)]),
    /// Aside callout for a tip or caveat.
    Note(&'static str),
    /// Monospace flow diagram (whitespace preserved), for showing how steps connect.
    Diagram(&'static str),
    /// Blocks that show on one [`Surface`] only, in this place: copy that differs while one client
    /// has a screen the other lacks. Read a guide through [`Guide::blocks_on`], which keeps them
    /// on their surface and drops them elsewhere.
    Only(Surface, &'static [Block]),
    /// Screenshot with alt text and an optional caption. `file` is its path under the guides'
    /// image folder (e.g. `"getting-started/01-create-deck.webp"`), its size [`image_size`].
    Image {
        /// The file, under the guides' image folder.
        file: &'static str,
        /// What the screenshot shows, for a screen reader.
        alt: &'static str,
        /// A line under it, if any.
        caption: Option<&'static str>,
    },
}

/// One guide: its address, title and summary, the subjects it covers, the guides it points on
/// to, and the article.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Guide {
    /// The guide's address on both sites, `/guides/<slug>/`.
    pub slug: &'static str,
    /// Its title, the page's heading.
    pub title: &'static str,
    /// One line under the title and on the index card.
    pub summary: &'static str,
    /// Subjects this guide covers (1-3), used for the index chip filter and the
    /// card chips. The first tag is the primary one (breadcrumb + JSON-LD).
    pub tags: &'static [&'static str],
    /// Slugs of related guides, linked from the detail page's "Related" block.
    pub related: &'static [&'static str],
    /// The article, top to bottom.
    pub blocks: &'static [Block],
}

/// Every guide, in the index's order.
pub static GUIDES: &[Guide] = &[
    Guide {
        slug: "getting-started",
        title: "Getting started with Zwipe",
        summary: "Zwipe builds Magic: The Gathering decks with swipes instead of forms. Here's the whole model.",
        tags: &["Getting started"],
        related: &[
            "deck-stats",
            "remove-cards",
            "budgeting",
            "land-targets",
            "import-export",
        ],
        blocks: &[
            Block::Lead(
                "Zwipe builds Magic: The Gathering decks by swiping through cards one at a time.",
            ),
            Block::H2("Create a deck"),
            Block::P(
                "Name a deck and pick a format. A commander format adds the command-zone fields: commander, plus partner, background, or signature spell where relevant.",
            ),
            Block::Image {
                file: "getting-started/01-create-deck.webp",
                alt: "The Create deck form with the Commander format picked and Krenko, Mob Boss as commander",
                caption: None,
            },
            Block::H2("Build by swiping"),
            Block::P(
                "Open the deck and tap the [[plus]] in its dock (on a phone, open Cards first), then act on the top card with a flick:",
            ),
            Block::Swipe(&[
                ("right", "add the card to your deck"),
                ("left", "skip it"),
                ("up", "send it to your maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::Image {
                file: "getting-started/02-add-cards.webp",
                alt: "The Add screen: a Goblin on top of the stack, the From row's Search chip, the bolt lit in the dock",
                caption: None,
            },
            Block::P(
                "Every add saves and syncs across your devices. A skip keeps that card out of this deck for a week, even after you close the app. Each deck keeps your place in the stack.",
            ),
            Block::H2("Shape the stack"),
            Block::P(
                "On a commander deck, Synergy, the [[bolt]] in the dock, starts the stack with cards that fit your commander. The [[funnel]] narrows by color, type, keyword, price, and more. Sorting reorders without changing which cards are eligible.",
            ),
            Block::H2("Review and refine"),
            Block::P(
                "The deck view shows live stats, charts, and warnings with one-tap fixes. Trim with the remove flow, set a budget or land target, import or export. Each has its own guide below.",
            ),
            Block::Image {
                file: "getting-started/03-deck-cards-2.webp",
                alt: "A built deck's card list: featured commander and MVP images up top, quick add, and the group and filter rows",
                caption: None,
            },
        ],
    },
    Guide {
        slug: "swipe-to-build",
        title: "Build a deck by swiping",
        summary: "The add-cards flow: right to add, left to skip, up to maybeboard, down to undo, with synergy and filters to shape the stack.",
        tags: &["Swiping", "Deck building"],
        related: &["remove-cards", "swipe-memory", "filtering"],
        blocks: &[
            Block::Lead("The Add screen deals cards as a stack, and you decide each with a flick."),
            Block::H2("The four swipes"),
            Block::P(
                "A first-visit hint (\"Swipe to build\") covers the gestures, and the `?` in the header reopens it.",
            ),
            Block::Swipe(&[
                ("right", "add the card to your deck"),
                ("left", "skip it"),
                ("up", "send it to your maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::P("Only the top card responds. Double-faced cards flip to show the back."),
            Block::Image {
                file: "swipe-to-build/01-swipe-hint.webp",
                alt: "The Swipe to build hint: the four colored directions and the Synergy note",
                caption: None,
            },
            Block::H2("Skips are remembered"),
            Block::P(
                "A left swipe keeps the card out of this deck's pile for a week. `Clear` beside Clear skips in the deck's Danger zone panel brings skipped cards back sooner. See Swipe memory.",
            ),
            Block::H2("Synergy"),
            Block::P(
                "On a commander deck, the [[bolt]] in the dock turns Synergy on and the stack starts with cards that fit your commander, best fits first. Off, you browse every legal card. See Synergy-ranked cards.",
            ),
            Block::Note(
                "If synergy is still warming up, Zwipe shows the full pool and tells you once.",
            ),
            Block::H2("Filter, sort, and the maybeboard"),
            Block::P(
                "The [[funnel]] decides which cards are dealt; a dot on it marks an active filter. Sorting only reorders. The Maybe chip after From deals your maybeboard instead; a right swipe there moves a card to the mainboard. Each source keeps its own filter.",
            ),
            Block::Image {
                file: "swipe-to-build/02-filter-sheet.webp",
                alt: "The filter sheet with Types picked and a mana value range set",
                caption: None,
            },
            Block::Image {
                file: "swipe-to-build/03-maybeboard-stack.webp",
                alt: "The From row switched to Maybe, its chip lit, dealing saved cards",
                caption: None,
            },
            Block::H2("Undo"),
            Block::P(
                "Swipe down to undo: an added card comes back out, a skipped card comes back. Your place and undo history stay with each deck while the app is open. The [[refresh]], or a filter change, deals a fresh stack and clears the history.",
            ),
            Block::Note(
                "At the end of the stack, swipe down on the empty card to step back through your last swipes.",
            ),
            Block::Note(
                "Swipe confirmations in Settings sets how a swipe is confirmed: `Chip` shows it in a small chip at the top left, `Toasts` pops up a message, `Off` says nothing.",
            ),
        ],
    },
    Guide {
        slug: "remove-cards",
        title: "Remove cards from a deck",
        summary: "The remove flow mirrors building: swipe through the cards already in your deck and trim.",
        tags: &["Swiping", "Deck building"],
        related: &["swipe-to-build", "swipe-memory"],
        blocks: &[
            Block::Lead("The remove flow deals only the cards already in your deck."),
            Block::H2("The four swipes"),
            Block::P("A first-visit hint (\"Swipe to trim\") covers them:"),
            Block::Swipe(&[
                ("right", "remove the card from your deck"),
                ("left", "keep it"),
                (
                    "up",
                    "move it between boards (to the maybeboard, or a maybeboard card up to the mainboard)",
                ),
                ("down", "undo your last swipe"),
            ]),
            Block::H2("Boards, filter, and refresh"),
            Block::P(
                "The board chips at the bottom left pick the pool: Main, Maybe, Side, or All. The [[funnel]] narrows it. The [[refresh]] restarts the stack and clears your undo history.",
            ),
            Block::Image {
                file: "remove-cards/01-remove-swipe.webp",
                alt: "The Remove screen mid-swipe, with the Main board chip picked",
                caption: None,
            },
            Block::Note(
                "A removed card stays out of this deck's add pile for a month (a skip lasts a week). Clear skips brings it back sooner.",
            ),
        ],
    },
    Guide {
        slug: "swipe-memory",
        title: "Swipe memory: how long skips last",
        summary: "A skip keeps a card out of that deck for a week and a removal for a month, even after you close the app. Clear skips starts a deck over.",
        tags: &["Swiping"],
        related: &["swipe-to-build", "remove-cards"],
        blocks: &[
            Block::Lead("Skip a card and it stays out of that deck's swipe pile for a week."),
            Block::H2("A skip lasts a week"),
            Block::P(
                "A left swipe on the add screen keeps the card out of the pile for 7 days, even after you close and reopen the app. Skip it again when it comes back and the 7 days start over. Quick add still finds it by name.",
            ),
            Block::H2("A removal lasts a month"),
            Block::P("Removing a card from a deck keeps it out of the pile for 30 days."),
            Block::H2("When cards come back"),
            Block::P(
                "When a skip or removal runs out, the card comes back at the end of the pile, after every card you haven't seen yet.",
            ),
            Block::H2("It's per deck"),
            Block::P("Skipping a card in one deck never hides it in another."),
            Block::H2("Pick up where you left off"),
            Block::P(
                "While the app is open, each deck keeps its place in the stack and your recent swipes for undo. Restarting the app, or changing the filter, deals a fresh stack.",
            ),
            Block::H2("Changed your mind? Clear skips"),
            Block::P(
                "Open the deck, scroll to the Danger zone panel and tap `Clear` beside Clear skips. Every card you skipped or removed for that deck comes back right away.",
            ),
            Block::Image {
                file: "swipe-memory/01-clear-skips.webp",
                alt: "The Danger zone panel with the Clear skips row and its Clear button",
                caption: None,
            },
            Block::Note("Clearing skips can't be undone, and it only affects the one deck."),
        ],
    },
    Guide {
        slug: "filtering",
        title: "Filter the card pool",
        summary: "Stack filters to control which cards you see: color, mana, type, text, keyword, rarity, set, price, and more.",
        tags: &["Filtering", "Cards"],
        related: &["card-roles", "oracle-tags"],
        blocks: &[
            Block::Lead(
                "Filters decide which cards you see. The sheet is an accordion, one section per attribute.",
            ),
            Block::H2("What you can filter on"),
            Block::Bullets(&[
                "`Name` and `Flavor text`: contains, or doesn't contain.",
                "`Oracle text`: rules text, oracle words, or keyword abilities, included or excluded.",
                "`Types`: the basic card types, or any type-line word like `legendary` or `dragon`.",
                "`Mana`: mana value, color identity (`Exact`, or a `Range` within a set of colors), and colors produced.",
                "`Combat`: power and toughness, `exact` or a `range`.",
                "`Rarity`, `Set`, `Artist`: include or exclude values.",
                "`Card roles`: 27 roles like `Ramp`, `Removal`, `Draw`, `Wipe`.",
                "`Oracle tags`: thousands of functional tags (spot removal, ramp, reanimation); include or exclude, with a curated set up front and search for the rest.",
                "`Format`: `Is commander in` and `Is legal in`.",
                "`Price`: min and max in `USD`, `EUR`, or `TIX`.",
                "`Sort`: the order cards come in.",
                "`Config`: `Is playable`, `Is digital`, `Is oversized`, `Is promo`, and `Has content warning`, each yes, no, or either.",
            ]),
            Block::Image {
                file: "swipe-to-build/02-filter-sheet.webp",
                alt: "The filter sheet mid-use: Types selected, mana value 2 to 5, red picked",
                caption: None,
            },
            Block::H2("Combining"),
            Block::P(
                "Sections combine with `AND`. A multi-value section's `Any`/`All` toggle sets whether a card needs one selected value or all of them. `include` and `exclude` can run at once.",
            ),
            Block::Note(
                "The same value in both `include` and `exclude` stops the [[check]] with \"Filter can't both include and exclude\" and the section's name; the sheet stays open so you can fix it.",
            ),
            Block::Image {
                file: "filtering/02-filter-hint.webp",
                alt: "The Filters hint: stack sections and apply with the check in the dock",
                caption: None,
            },
            Block::H2("Sorting"),
            Block::P(
                "Sort by `Name`, `Mana value`, `Power`, `Toughness`, `Rarity`, `Release Date`, `Price (USD)`, `Price (EUR)`, `Price (TIX)`, `Popularity (EDHREC)`, or `Random`. Sorting only reorders. Each screen keeps its own filter and sort for the session.",
            ),
            Block::H2("Defaults and Reset"),
            Block::P(
                "Each screen has a default view, and the [[eraser]] on the filter sheet returns to it:",
            ),
            Block::Bullets(&[
                "Add cards on a commander deck: your commander's colors, the format's legal cards, in synergy order. Once you hit your land target, lands drop out too.",
                "Add cards on other formats: the format's legal cards, in default order.",
                "Pick a commander: every eligible commander, in popularity order.",
                "Remove and browse: an empty filter, over your deck's own cards.",
            ]),
            Block::P(
                "The dot on the [[funnel]] means you've changed a filter or sort from that default. Synergy is a separate mode and stays however you set it.",
            ),
            Block::Note("The built-in defaults never light the dot."),
            Block::H2("Card roles"),
            Block::P(
                "A card can hold several roles (`Lightning Bolt` is `Burn` and `Removal`). Zwipe assigns them from the card's Oracle tags; you pick the roles you want. See Read a card at a glance.",
            ),
        ],
    },
    Guide {
        slug: "organize-and-browse",
        title: "Group, filter, and sort your decks",
        summary: "The deck list sorts your decks into sections with Group by, narrows them with Show, and orders them with Sort.",
        tags: &["Deck building"],
        related: &["deck-card-list", "deck-tags", "filtering"],
        blocks: &[
            Block::Lead("Group by, Show, and Sort keep a long deck list easy to read."),
            Block::H2("Group by"),
            Block::P(
                "Group by sections your decks by Format, Color, or Tag; a deck with several tags appears under each. None is one collapsible group headed All. Color makes a group per exact color combination, headed by its mana pips.",
            ),
            Block::Image {
                file: "organize-and-browse/01-deck-list-by-tag.webp",
                alt: "The deck list grouped by Tag, with the Show pips and tag chips above",
                caption: None,
            },
            Block::H2("Show and Sort"),
            Block::P(
                "Show narrows the list: color pips keep decks whose identity contains every pip you pick, tag chips match any tag, and All resets. Sort orders the decks by Name, Size, Format, Last changed, or Price, and its last chip flips the direction.",
            ),
            Block::H2("Collapse what you're done with"),
            Block::P("Tap a group header to fold it. Collapses last for the visit."),
            Block::Note(
                "A deck's own card list has its own controls. See Work a deck's card list.",
            ),
        ],
    },
    Guide {
        slug: "deck-card-list",
        title: "Work a deck's card list",
        summary: "A deck's cards as a list or stacks: group and narrow them, then open a row to star, flag, count, move, or send a card.",
        tags: &["Deck building", "Cards"],
        related: &[
            "organize-and-browse",
            "deck-mvps",
            "filtering",
            "keyboard-shortcuts",
        ],
        blocks: &[
            Block::Lead(
                "A deck's card list is where you read the deck and fix it one card at a time.",
            ),
            Block::H2("The rows above the list"),
            Block::Bullets(&[
                "View shows the cards as a List or as Stacks.",
                "Group by sections them by Type, Mana value, Color, or Card role.",
                "Boards is three independent toggles, Main, Maybe, and Side; turn the last one off and Main snaps back on.",
                "Show toggles Tokens, the Command zone, and Art, and Flagged narrows the list to the cards you flagged.",
            ]),
            Block::P(
                "The [[funnel]] in the dock filters the list like the swipe screens' filter. See Filter the card pool.",
            ),
            Block::H2("Collapse what you're done with"),
            Block::P(
                "Tap a group header to fold it; Tokens, Lands, Maybeboard, and Sideboard fold too. The last button in the Show row folds every group at once. Collapses last for the visit.",
            ),
            Block::Image {
                file: "organize-and-browse/02-cards-by-type-collapsed-2.webp",
                alt: "A card list grouped by Type with the Creatures section collapsed",
                caption: None,
            },
            Block::H2("A card's row"),
            Block::P("Tap a row to open it. Under the card's details are its actions:"),
            Block::Bullets(&[
                "[[image]] shows the card full size.",
                "[[send]] sends it to another of your decks, on the board you pick.",
                "[[star]] makes it one of the deck's three MVPs. See Star your deck's MVPs.",
                "[[flag]] flags it, for the Flagged chip to find later.",
                "[[minus]] and [[plus]] change how many you run.",
                "`Printings` swaps the printing.",
                "The board buttons move it to the `Mainboard`, `Maybeboard`, or `Sideboard`.",
            ]),
            Block::H2("Quick add"),
            Block::P(
                "Quick add at the top of the card list searches by name and adds a result to your mainboard with one tap. It reaches cards you skipped while swiping.",
            ),
            Block::Image {
                file: "organize-and-browse/03-quick-add-results-2.webp",
                alt: "Quick add results floating over the card list",
                caption: None,
            },
        ],
    },
    Guide {
        slug: "card-search",
        title: "Search for a card",
        summary: "Look any card up by name, signed in or out, and open its page: printings, how decks use it, and Send to.",
        tags: &["Cards"],
        related: &["deck-card-list", "keyboard-shortcuts"],
        blocks: &[
            Block::Lead(
                "Card search finds any Magic card by name and opens its card page, signed in or out.",
            ),
            Block::H2("Where it is"),
            Block::P(
                "On a wide window it's the [[search]] and the Card search box in the nav, on app.zwipe.net and on zwipe.net. On a phone, in the app and on both sites, tap the [[search]] at the top of the screen and the box opens across the header. On the web, `/` puts the cursor in it.",
            ),
            Block::Image {
                file: "card-search/01-search-chips.webp",
                alt: "The card search open across a phone's header, its first matches as chips under the box",
                caption: None,
            },
            Block::H2("Searching"),
            Block::Bullets(&[
                "Type part of a name. The first 10 matches show as chips under the box; tap one to open its card.",
                "Punctuation between words counts as a space, so `fire ice` finds Fire // Ice.",
                "Enter opens the card when only one matches. With several it opens Cards matching your text, a page of every match, each opening its card.",
                "The ✕ empties the box.",
            ]),
            Block::Note("On zwipe.net a pick opens the card on app.zwipe.net."),
            Block::H2("The card page"),
            Block::P(
                "A card page shows the card and its details, with its card roles and prices, and how Zwipe's players use it:",
            ),
            Block::Bullets(&[
                "Decks: how many decks run it, and how many star it as an MVP.",
                "Swipes: how often it was dealt, and how often it was added, skipped, saved to a maybeboard, or added and later removed.",
                "As a commander, for a card that can lead a Commander deck: the decks it leads, and how often it was picked when offered.",
            ]),
            Block::Image {
                file: "card-search/02-card-page.webp",
                alt: "A card page on a phone: the card, its details and Printings, and its Decks and Swipes tables",
                caption: None,
            },
            Block::P(
                "When several cards share a name, the page shows them all, each opening its own page.",
            ),
            Block::H2("Printings, Send to, and Share"),
            Block::Bullets(&[
                "`Printings` steps through the card's printings, each with its set, artist, and prices. On the web, `p` opens it.",
                "[[send]] adds the card to one of your decks: pick a board, then a deck. Zwipe says Already in when the deck has it. On the web, Shift+P opens it.",
                "[[share]] copies the card's link.",
            ]),
            Block::Note("Signed out, Create account takes Send to's place in the dock."),
        ],
    },
    Guide {
        slug: "synergy",
        title: "Synergy-ranked cards",
        summary: "On a commander deck, Synergy deals the cards that work with your commander first, best fits first, then keeps going with cards like your deck.",
        tags: &["Commander", "Cards"],
        related: &["commander-and-formats", "oracle-tags"],
        blocks: &[
            Block::Lead(
                "On a commander deck, Synergy leads with the cards that work with your commander.",
            ),
            Block::H2("Where and how"),
            Block::P(
                "Synergy is the [[bolt]] in the Add screen's dock, there when the source is Search and the deck has a commander. On, the stack starts with fitting cards, best first, 25 at a time, shuffled within each batch per deck per day. Off, you browse everything legal. It's on by default, and toggling it reloads the stack.",
            ),
            Block::Image {
                file: "synergy/01-synergy-on.webp",
                alt: "The add screen with the Synergy link lit and a Goblin on top for a Goblin commander",
                caption: None,
            },
            Block::Note(
                "If a commander's synergy isn't ready yet, Zwipe shows all cards and says \"Synergy warming up\" once.",
            ),
            Block::H2("When the synergy picks run out"),
            Block::P(
                "Each commander has a few hundred synergy picks. After them, the stack continues with cards like your deck, where a rare shared trait counts for more than a common one like ramp. Zwipe says so once. With a sort applied, the stack stops at the last synergy pick.",
            ),
            Block::H2("Synergy vs. sorting"),
            Block::P(
                "Synergy sets which cards appear and their order; Sort only reorders the current set.",
            ),
            Block::H2("Oracle tags sharpen the order"),
            Block::P(
                "Oracle tags selected on the deck shape the order too. See Sharpen suggestions with Oracle tags.",
            ),
        ],
    },
    Guide {
        slug: "commander-and-formats",
        title: "Choose a commander and format",
        summary: "Pick a format, then a commander. Zwipe enforces each format's rules and your commander's color identity.",
        tags: &["Commander", "Deck building"],
        related: &["synergy", "commander-maybeboard"],
        blocks: &[
            Block::Lead(
                "Your format sets the rules, your commander sets the colors, and Zwipe keeps the build inside both.",
            ),
            Block::H2("Pick a format"),
            Block::P(
                "Tap a format to see its details (pool, size, life, command zone); tap again to clear. Commander formats are listed first.",
            ),
            Block::Bullets(&[
                "Commander formats: `Brawl`, `Commander`, `Duel Commander`, `Historic Brawl`, `Oathbreaker`, `Pauper Commander`, `PreDH`, `Standard Brawl`.",
                "Other formats: `Alchemy`, `Explorer`, `Future`, `Gladiator`, `Historic`, `Legacy`, `Modern`, `Old School`, `Pauper`, `Penny Dreadful`, `Pioneer`, `Premodern`, `Standard`, `Timeless`, `Vintage`.",
            ]),
            Block::P(
                "Commander is 100 cards, singleton, 40 life; Oathbreaker is 60 with a planeswalker and signature spell; 60-card constructed formats allow four copies. Changing the format clears your commander.",
            ),
            Block::Image {
                file: "commander-and-formats/01-format-picker.webp",
                alt: "The Format picker with Commander selected",
                caption: None,
            },
            Block::H2("Pick a commander"),
            Block::P(
                "Each command-zone field (Commander, Partner, Background, Signature spell) fills two ways:",
            ),
            Block::Steps(&[
                "Type to search. Filter on limits results to eligible cards; off searches any card by name.",
                "Tap Swipe for the Swipe select screen, most-played first: right to choose, left to skip, up to save it for later, down to undo.",
            ]),
            Block::P(
                "Partner and Background appear only when your commander supports them. Oathbreaker needs a signature spell within the planeswalker's colors.",
            ),
            Block::Image {
                file: "commander-and-formats/02-swipe-select.webp",
                alt: "The Swipe select pile with a legendary creature on top",
                caption: None,
            },
            Block::Note(
                "Pick a \"Partner with [Name]\" commander and Zwipe fills in the named partner.",
            ),
            Block::H2("Color identity"),
            Block::P(
                "In commander formats, the deck's colors come from the commander, partner, and background. The add screen stays inside them, and off-color cards show in warnings.",
            ),
        ],
    },
    Guide {
        slug: "commander-maybeboard",
        title: "Save commanders for later",
        summary: "One account-wide list of commanders you might build someday: swipe up to save, then start decks straight from it.",
        tags: &["Commander", "Deck building"],
        related: &[
            "commander-and-formats",
            "commander-suggestions",
            "swipe-to-build",
        ],
        blocks: &[
            Block::Lead(
                "Swipe up while picking a commander and it lands on your commander maybeboard, one list shared across your account.",
            ),
            Block::H2("Two maybeboards, one word"),
            Block::P(
                "Each deck has its own maybeboard for cards. The commander maybeboard belongs to your account and holds commanders only.",
            ),
            Block::H2("Save a commander"),
            Block::P(
                "In any Swipe select picker for a commander, the up swipe saves the top card:",
            ),
            Block::Swipe(&[
                ("right", "choose it for this deck"),
                ("left", "skip it"),
                ("up", "save it to your commander maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::Image {
                file: "commander-maybeboard/01-swipe-select-hint.webp",
                alt: "The Swipe select hint: right chooses, up saves to your commander maybeboard",
                caption: None,
            },
            Block::P(
                "The pile advances and you keep swiping. A down swipe takes the save back. The list holds up to 50 commanders.",
            ),
            Block::Only(
                Surface::App,
                &[
                    Block::H2("The Commander maybeboard screen"),
                    Block::P(
                        "Open Commander maybeboard from the menu. Saves show as card rows with art (the Art chip hides it), newest first; Sort orders them by Newest saved, Name, Mana value, or Color, and its last chip flips the direction. Expand a row for details and four actions:",
                    ),
                    Block::Image {
                        file: "commander-maybeboard/02-saved-commanders.webp",
                        alt: "The Commander maybeboard screen: saved commanders with power level, price, and color pips",
                        caption: None,
                    },
                    Block::Bullets(&[
                        "[[image]] shows the card full size.",
                        "`Printings` swaps the printing, and your pick carries into Create deck.",
                        "`Create deck` starts a new deck with that commander already in the command zone and the format set to Commander.",
                        "`Remove` takes it off the list.",
                    ]),
                    Block::Image {
                        file: "commander-maybeboard/03-expanded-row.webp",
                        alt: "A saved commander expanded, with Image, Printings, Create deck, and Remove",
                        caption: None,
                    },
                    Block::H2("Add without swiping"),
                    Block::P(
                        "Quick add searches commanders by name; tap a result to save it. The color pips and the `Filter` button narrow the list of saves.",
                    ),
                    Block::Image {
                        file: "commander-maybeboard/04-maybeboard-hint.webp",
                        alt: "The Commander maybeboard hint: Swipe, Quick add, and Create deck",
                        caption: None,
                    },
                    Block::H2("Swipe from the list"),
                    Block::P(
                        "The Swipe button sits on the Commanders panel, even with nothing saved yet, and deals commanders you haven't saved, most played first:",
                    ),
                    Block::Swipe(&[
                        ("left", "skip it"),
                        ("up", "save it to your commander maybeboard"),
                        ("right", "start a new deck with it"),
                        ("down", "undo your last swipe"),
                    ]),
                    Block::Note(
                        "Clear the whole list from the Danger zone panel: tap `Clear` beside Clear maybeboard. It asks first; there's no undo.",
                    ),
                ],
            ),
            Block::Only(
                Surface::Web,
                &[
                    Block::H2("The Commanders screen"),
                    Block::P(
                        "Tap Commanders in the nav. It holds two lists side by side, stacked on a phone: Maybeboard, the commanders you save, and Suggested, the ones that fit your decks. Each list has its own Show row and Swipe. The [[funnel]] at the bottom right narrows both lists at once.",
                    ),
                    Block::H2("Maybeboard"),
                    Block::P(
                        "Saves show as card rows with art; Art in the Show row hides it. Quick add saves a commander by name, and Sort changes the order. Expand a row for details and four actions:",
                    ),
                    Block::Bullets(&[
                        "[[image]] shows the card full size.",
                        "`Printings` swaps the printing, and your pick carries into Create deck.",
                        "`Create deck` starts a new deck with that commander already in the command zone and the format set to Commander.",
                        "`Remove` takes it off the list.",
                    ]),
                    Block::H2("Swipe from the Maybeboard"),
                    Block::P(
                        "The Maybeboard's Swipe deals commanders you haven't saved, most played first:",
                    ),
                    Block::Swipe(&[
                        ("right", "start a new deck with it"),
                        ("left", "skip it"),
                        ("up", "save it to your maybeboard"),
                        ("down", "undo your last swipe"),
                    ]),
                    Block::Note(
                        "`Clear maybeboard` empties the saved list; it asks first and there's no undo. Suggested, the other list, has its own guide: Suggested commanders.",
                    ),
                ],
            ),
        ],
    },
    Guide {
        slug: "playstyle",
        title: "Playstyle",
        summary: "What your decks say about how you play: your colors, card roles, Oracle tags and staples.",
        tags: &["Commander"],
        related: &["commander-suggestions", "synergy", "activity"],
        blocks: &[
            Block::Lead(
                "Playstyle reads every deck you have, in any format, and shows what they have in common.",
            ),
            Block::H2("Finding it"),
            Block::Only(
                Surface::App,
                &[Block::Bullets(&[
                    "On the Home screen, tap Playstyle on the Colors panel. It's also in the menu.",
                    "Your colors, card roles and Oracle tags come first, then Suggested commanders. See Suggested commanders.",
                    "Staples, at the bottom, lists the cards in the most of your decks: each row shows the share of decks that run it, and an expanded row the count, like 4 of 6. Basic lands are left out. Type and Color chips narrow it, and it has a `?` too.",
                ])],
            ),
            Block::Only(
                Surface::Web,
                &[Block::Bullets(&[
                    "On the Home screen, tap Playstyle on the Colors panel. It's also in the menu.",
                    "Your colors, card roles and Oracle tags come first.",
                    "Staples lists the cards in the most of your decks: each row shows the share of decks that run it, and an expanded row the count, like 4 of 6. Basic lands are left out. Type and Color chips narrow it, and it has a `?` too.",
                    "Suggested commanders are on the Commanders screen. See Suggested commanders.",
                ])],
            ),
            Block::H2("What your decks say"),
            Block::P(
                "Colors has a bar per color with its mana pip and how many of your cards are that color, colorless last. It's the same panel as on Home. Each section folds; tap its header to open or close it.",
            ),
            Block::Image {
                file: "playstyle/01-colors.webp",
                alt: "Playstyle's Colors panel: a bar per color with its card count, colorless last, over the Show row of color chips",
                caption: None,
            },
            Block::Bullets(&[
                "Summary shows how many decks were read, your most built color combination and your top single color.",
                "Card roles shows how many of your decks fill each role, like ramp or removal. Tap a role to see its tags.",
                "Oracle tags lists your 10 most impactful tags. Tap a tag to see what it means.",
                "Colors, Card roles and Oracle tags each have a `?` in their header.",
            ]),
            Block::H2("How a tag's impact adds up"),
            Block::Bullets(&[
                "Decks is how many of your decks have the tag, Cards how many cards carry it across them. More of either, more impact.",
                "Impact is how much more your decks lean on a tag than Magic cards do overall. Your top tag scores 100.",
                "A tag your decks use no more than Magic overall doesn't show up, and with two or more decks a tag needs to be in at least 2 of them.",
                "Each column has its own `?`, too.",
            ]),
            Block::Image {
                file: "playstyle/02-oracle-tag-impact-2.webp",
                alt: "Oracle tags: each tag's Decks, Cards and Impact, the top tag at 100",
                caption: None,
            },
            Block::Note(
                "You need a deck with some cards in it before anything shows up. Any format counts.",
            ),
        ],
    },
    Guide {
        slug: "commander-suggestions",
        title: "Suggested commanders",
        summary: "Commanders you haven't built yet, ranked against every deck you have: see why each one fits, then start a deck, save it, or swipe the ranking.",
        tags: &["Commander", "Deck building"],
        related: &["playstyle", "commander-maybeboard", "commander-and-formats"],
        blocks: &[
            Block::Lead(
                "Suggested commanders ranks commanders you haven't built yet against every deck you have, in any format, best fit first.",
            ),
            Block::H2("Finding it"),
            Block::Only(
                Surface::App,
                &[
                    Block::P(
                        "Open Playstyle from the Colors panel on Home or from the menu. Suggested commanders sits under your colors, card roles and Oracle tags: your top 10 as card rows, best fit first. Its `?` explains the ranking.",
                    ),
                    Block::P(
                        "Tap a color identity in the Show row to narrow the list to it; tap again to clear.",
                    ),
                ],
            ),
            Block::Only(
                Surface::Web,
                &[
                    Block::P(
                        "Tap Commanders in the nav. Suggested sits beside your Maybeboard, under it on a phone: the 10 commanders that fit your decks best, and Show more adds 10 more, once. Its `?` explains the ranking, and the [[funnel]] at the bottom right narrows it along with the Maybeboard.",
                    ),
                    Block::P(
                        "Until you've built a deck, Suggested shows popular commanders instead.",
                    ),
                ],
            ),
            Block::Image {
                file: "commander-suggestions/01-suggested-list.webp",
                alt: "Suggested commanders as card rows, best fit first, under the Show row of color identities",
                caption: None,
            },
            Block::H2("Why this commander"),
            Block::P(
                "Expand a row and tap Why this commander? for the Oracle tags it shares with your decks and the colors you keep building in. While you swipe, the card's details sheet shows the same reasons.",
            ),
            Block::Image {
                file: "playstyle/03-why-this-commander.webp",
                alt: "An expanded suggestion's Why this commander?: the Oracle tags it shares with your decks and a color you keep building in",
                caption: None,
            },
            Block::H2("What you can do with one"),
            Block::Bullets(&[
                "`Create deck` starts a new deck with it as your commander.",
                "`Maybeboard` saves it to your commander maybeboard. See Save commanders for later.",
                "`Remove` hides it for a week and pulls the next one up.",
            ]),
            Block::H2("Swipe the ranking"),
            Block::Only(
                Surface::App,
                &[Block::P(
                    "Swipe deals the same ranking as a stack, up to 100 deep. Tap the [[funnel]] while you swipe to narrow it to one color identity.",
                )],
            ),
            Block::Only(
                Surface::Web,
                &[Block::P(
                    "Suggested's Swipe deals the ranking as a stack, best fit first, and keeps dealing as you swipe.",
                )],
            ),
            Block::Swipe(&[
                ("right", "start a new deck with this commander"),
                ("left", "skip it for a week"),
                ("up", "save it to your commander maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::Only(
                Surface::App,
                &[Block::P(
                    "Skipped commanders come back after seven days, or sooner with `Clear` beside Clear skips in the Danger zone panel.",
                )],
            ),
            Block::Only(
                Surface::Web,
                &[Block::P(
                    "Skipped commanders come back after seven days, or sooner with `Clear` beside Clear skips in the Danger zone under Suggested.",
                )],
            ),
        ],
    },
    Guide {
        slug: "activity",
        title: "Activity",
        summary: "How you pick, read from your swipes: a year of activity, what your swipes came to, your rates and the cards you keep passing on.",
        tags: &["Swiping"],
        related: &["playstyle", "swipe-memory"],
        blocks: &[
            Block::Lead(
                "Playstyle reads the decks you build. Activity reads how you pick: every card you add, skip, save or take out.",
            ),
            Block::H2("Finding it"),
            Block::P(
                "On the Home screen, tap Activity under the counters. Each section has a `?`.",
            ),
            Block::Image {
                file: "activity/01-heatmap-and-totals-2.webp",
                alt: "The Activity screen: the counters, a year of days as a heatmap, and Totals for Added, Skipped, Maybeboard and Removed",
                caption: None,
            },
            Block::H2("What's on it"),
            Block::Bullets(&[
                "Activity is a heatmap of the last year, a square per day, darker the more you swiped and searched. Tap a square for its counts. Totals, under it, counts the cards dealt to your decks: added, skipped, saved to a maybeboard, or added and later removed.",
                "Card roles and Oracle tags share a Show chip row: Added, Skipped and Maybeboard are shares of the cards dealt to you, and Removed is the share of cards you added and later took out. Pick one in either section and both follow.",
                "Card roles shows your rate for each role. Tap one to see the tags you swiped under it.",
                "Oracle tags lists up to ten tags where your rate is higher than other players', highest first. Tap a tag to see what it means.",
                "Both read your decks with a commander, for roles and tags dealt at least 10 times.",
                "Most skipped lists cards you skip and never add or save, with the skip count and, in an open row, the commanders you skipped it under. Skips are weighed against other players': a card most people pass on needs a lot of skips to show up, one they usually keep needs two.",
            ]),
            Block::Image {
                file: "activity/02-card-roles-2.webp",
                alt: "Card roles with Added picked in the Show row: each role's share of the cards dealt to you",
                caption: None,
            },
            Block::Image {
                file: "activity/03-most-skipped-2.webp",
                alt: "Most skipped: cards you keep passing on, each with its skip count",
                caption: None,
            },
        ],
    },
    Guide {
        slug: "budgeting",
        title: "Set a budget with price targets",
        summary: "Give a deck a price target and Zwipe tracks the running total, warns as you approach it, and offers buy links.",
        tags: &["Deck building"],
        related: &["land-targets", "deck-stats"],
        blocks: &[
            Block::Lead(
                "Give a deck a price target and Zwipe tracks the running total as you build.",
            ),
            Block::H2("Set a budget"),
            Block::P(
                "On the edit form, pick a currency (USD, EUR, or TIX) and an amount under Price target; leave it blank for no budget. It shows in the deck's Budget section.",
            ),
            Block::H2("As it fills up"),
            Block::P(
                "Zwipe alerts you once as the deck crosses 50%, 75%, and 100% of the budget (for example, \"Deck at 82% of your $50.00 budget\"), and a standing warning appears if you go over.",
            ),
            Block::Image {
                file: "budgeting/01-budget-section.webp",
                alt: "The Budget section with the currency chips and a total past its price target",
                caption: None,
            },
            Block::Note(
                "Cards with no price in your currency count as zero, so the total is a floor, not an appraisal.",
            ),
            Block::H2("Prices and buying"),
            Block::P(
                "The Budget section shows Total price and Average card price; the currency chip in its header switches USD, EUR, and TIX. Each card lists its prices while you swipe. The cart in the deck's action dock sends the whole deck to TCGplayer or Card Kingdom, and each stack's cart sends that stack's cards.",
            ),
        ],
    },
    Guide {
        slug: "land-targets",
        title: "Set a land target",
        summary: "Tell Zwipe how many lands you want and it warns you as the deck fills, so you don't finish short on mana.",
        tags: &["Deck building"],
        related: &["budgeting", "deck-stats"],
        blocks: &[
            Block::Lead("Set how many lands you want and Zwipe warns you as the deck fills."),
            Block::H2("Set the target"),
            Block::P(
                "Once the deck has a format, the Land target stepper shows the format's default, `37 (default)` for 100-card formats and `17 (default)` for 60-card ones. Step it to set your own, and × clears it. With no format it reads `Not set`.",
            ),
            Block::H2("Feedback"),
            Block::P(
                "Cross the target while editing and Zwipe toasts \"Land target reached\" or \"Below land target\" with your target, like \"Land target reached (37)\", and a standing warning shows whenever you're under.",
            ),
            Block::Image {
                file: "land-targets/01-mana-section.webp",
                alt: "The Mana section: lands against their target, average mana value, the curve, and cost fulfillment",
                caption: None,
            },
            Block::H2("Lands leave the stack"),
            Block::P(
                "Open the add screen at or past target (yours, or the format default) and lands drop out of the stack: `Land` goes into the `Basic types` exclude filter. Remove that exclude to bring lands back; `Reset` keeps it, since it's the default at target. Reaching the target mid-session takes effect the next time you enter the screen.",
            ),
            Block::Note("With no target set, warnings use your format's default land count."),
        ],
    },
    Guide {
        slug: "deck-tags",
        title: "Tag decks by archetype",
        summary: "Label a deck's game plan with up to five strategy tags: Aggro, Tokens, Reanimator, and over a hundred more.",
        tags: &["Deck building", "Oracle tags"],
        related: &["tags-roles-and-oracle-tags", "oracle-tags"],
        blocks: &[
            Block::Lead(
                "Tags label a deck's game plan: Aggro, Tokens, Reanimator, Stax, and over a hundred more.",
            ),
            Block::H2("Adding tags"),
            Block::P(
                "Open Tags from the edit form, then tap to add or remove; each tag shows its definition up top. Search by name, and use × to clear them all.",
            ),
            Block::Image {
                file: "deck-tags/01-tag-picker.webp",
                alt: "The Deck tags picker at five of five, the tapped tag's definition above the list",
                caption: None,
            },
            Block::H2("How many"),
            Block::P(
                "Up to five per deck (shown as N/5), from one alphabetical list. Tags are never renamed, so old ones keep working.",
            ),
            Block::H2("Tags seed Oracle tags"),
            Block::P(
                "Picking a deck tag auto-selects the Oracle tags that define that archetype; Aggro pulls in haste and burn. Those Oracle tags sharpen the cards Zwipe suggests. See Sharpen suggestions with Oracle tags.",
            ),
        ],
    },
    Guide {
        slug: "oracle-tags",
        title: "Sharpen suggestions with Oracle tags",
        summary: "Pick the specific things your deck does, spot removal, ramp, reanimation, and Zwipe suggests cards that fit them better.",
        tags: &["Oracle tags", "Cards"],
        related: &[
            "oracle-tag-dictionary",
            "card-roles",
            "tags-roles-and-oracle-tags",
        ],
        blocks: &[
            Block::Lead(
                "Oracle tags say what a card does: `spot-removal`, `ramp`, `sacrifice-outlet`, and thousands more. Select them on a deck and Zwipe suggests cards that fit.",
            ),
            Block::H2("Where they come from"),
            Block::P(
                "Picking a deck tag (Aggro, Reanimator, Voltron) auto-selects a starter set for that archetype. Open Tags on the edit form to add or remove Oracle tags directly.",
            ),
            Block::H2("Selecting them"),
            Block::P(
                "The Oracle tags field shows a curated set of the most useful tags up front, and search reaches the rest, about 4,600 in all. A deck can hold up to 30.",
            ),
            Block::Image {
                file: "oracle-tags/01-tag-search.webp",
                alt: "The Oracle tags picker mid-search, with the selected tags outlined",
                caption: None,
            },
            Block::Note(
                "Tap Dictionary in the picker to look a tag up. See Browse the Oracle tags dictionary.",
            ),
            Block::H2("What they do"),
            Block::P(
                "Selected tags sharpen which cards Zwipe suggests while you swipe, and on a commander deck they shape the synergy order. See Synergy-ranked cards.",
            ),
            Block::Image {
                file: "oracle-tags/02-tags-section.webp",
                alt: "The deck view's Tags section: deck tags, the Oracle tags they seeded, and other tags below",
                caption: None,
            },
            Block::Note(
                "Deck tags are the quick start, Oracle tags the fine-tune. See Deck tags, card roles, and Oracle tags: how they fit.",
            ),
        ],
    },
    Guide {
        slug: "oracle-tag-dictionary",
        title: "Browse the Oracle tags dictionary",
        summary: "A reference for every Oracle tag: browse A to Z or search by name and description, see example cards, and use a tag on the spot.",
        tags: &["Oracle tags"],
        related: &["oracle-tags", "card-roles"],
        blocks: &[
            Block::Lead(
                "The Oracle tags dictionary describes every Oracle tag, about 4,600 in all.",
            ),
            Block::H2("Opening it"),
            Block::P(
                "In the Oracle tags picker (Tags on the deck's edit form), tap Dictionary next to Search; the picker's `?` has a Dictionary button too. On the filter sheet, the Oracle tags section has one for include and one for exclude.",
            ),
            Block::H2("Browse by letter"),
            Block::P(
                "An A to Z rail runs across the top, plus # for tags that start with a number. Tap a letter to list its tags alphabetically.",
            ),
            Block::Image {
                file: "oracle-tag-dictionary/01-by-letter.webp",
                alt: "The dictionary on the letter S, each entry with its description and parent tag",
                caption: None,
            },
            Block::H2("Search"),
            Block::P(
                "Search matches the whole catalog by tag name or description. A broad search shows the first matches.",
            ),
            Block::Image {
                file: "oracle-tag-dictionary/02-search.webp",
                alt: "A dictionary search for burn, listing the tag and its relatives",
                caption: None,
            },
            Block::H2("What each entry shows"),
            Block::P(
                "Each row lists the tag's name, description, and parent tags, then two buttons: Examples shows cards that carry the tag, and Use adopts it into the deck or filter you came from. A tag added since the last pass may read \"No description yet.\"",
            ),
            Block::Note("See Sharpen suggestions with Oracle tags for picking tags on a deck."),
        ],
    },
    Guide {
        slug: "card-roles",
        title: "Read a card at a glance",
        summary: "Every card shows chips for the roles it fills, Removal, Ramp, Card advantage, and so on, drawn straight from its Oracle tags.",
        tags: &["Cards", "Oracle tags"],
        related: &["oracle-tags", "filtering"],
        blocks: &[
            Block::Lead(
                "Card roles sum up a card's Oracle tags in broad labels like Removal, Ramp, or Card advantage.",
            ),
            Block::H2("The chips"),
            Block::P(
                "Expand a card's row for its Card roles chips. Roles are computed from the card's Oracle tags; you never pick them.",
            ),
            Block::H2("Drilling down"),
            Block::P(
                "Tap a role chip to see the Oracle tags under it. A trailing \"Other tags\" chip holds tags that fall under no role.",
            ),
            Block::Image {
                file: "card-roles/01-role-chips-2.webp",
                alt: "An expanded card row with its Card roles chips, one opened to its Oracle tags",
                caption: None,
            },
            Block::H2("Roles"),
            Block::P(
                "The deck view's Distributions section has a Card roles chart, the same roles counted across the whole deck. Tap a role to see the Oracle tags your cards carry under it.",
            ),
        ],
    },
    Guide {
        slug: "tags-roles-and-oracle-tags",
        title: "Deck tags, card roles, and Oracle tags: how they fit",
        summary: "One mental model for Zwipe tagging: deck tags seed Oracle tags, Oracle tags sharpen suggestions, card roles show the read.",
        tags: &["Oracle tags", "Cards"],
        related: &["deck-tags", "card-roles", "oracle-tags"],
        blocks: &[
            Block::Lead(
                "Deck tags seed Oracle tags, Oracle tags sharpen suggestions, and card roles sum up what a card's Oracle tags say.",
            ),
            Block::H2("Deck tags: the archetype you pick"),
            Block::P(
                "Deck tags label your game plan: Aggro, Tokens, Reanimator. Picking one auto-selects the Oracle tags that define it.",
            ),
            Block::H2("Oracle tags: the specific things your deck does"),
            Block::P(
                "Oracle tags are specific: `spot-removal`, `ramp`, `sacrifice-outlet`. Add or remove them directly to fine-tune. Selected Oracle tags sharpen which cards Zwipe suggests.",
            ),
            Block::Image {
                file: "tags-roles-and-oracle-tags/01-deck-and-oracle-tags.webp",
                alt: "A deck's Tags section: its deck tags over the Oracle tags they selected",
                caption: None,
            },
            Block::H2("Card roles: the summary"),
            Block::P(
                "You never assign card roles; they're computed from a card's Oracle tags and shown as chips (Removal, Ramp, Card advantage) when you expand a card's row or open its rules. Tap a role to see the Oracle tags under it. You can filter the stack by role.",
            ),
            Block::Image {
                file: "tags-roles-and-oracle-tags/02-role-to-oracle-tags-2.webp",
                alt: "A card's row expanded: its card roles, with Ramp tapped to show the Oracle tags under it",
                caption: None,
            },
            Block::H2("How it flows"),
            Block::Diagram(
                "You configure\n  Deck tags\n     │ seed\n     ▼\n  Oracle tags  (refine freely)\n     │ match\n     ▼\nZwipe serves\n  cards that fit your deck",
            ),
            Block::Note(
                "See Tag decks by archetype, Sharpen suggestions with Oracle tags, and Read a card at a glance.",
            ),
        ],
    },
    Guide {
        slug: "deck-mvps",
        title: "Star your deck's MVPs",
        summary: "Star up to three cards that define a deck. Stars show at a glance, steer that deck's suggestions, and travel when you clone.",
        tags: &["Deck building", "Cards"],
        related: &["deck-tags", "deck-stats", "share-your-deck"],
        blocks: &[
            Block::Lead(
                "Star up to three cards as a deck's MVPs and Zwipe leans that deck's suggestions toward them.",
            ),
            Block::H2("Star a card"),
            Block::P(
                "In a deck's card list, expand a card's row and tap [[star]]. A gold ★ marks the row; tapping [[star]] again frees the slot.",
            ),
            Block::Image {
                file: "deck-mvps/01-featured-strip-2.webp",
                alt: "A deck's featured strip: the commander beside three starred MVPs",
                caption: None,
            },
            Block::H2("Three per deck"),
            Block::P(
                "A fourth star is refused (\"This deck already has 3 MVPs\"); unstar one first.",
            ),
            Block::H2("What a star does"),
            Block::P(
                "MVPs steer this deck's suggestions. Stars are per deck; one deck's MVPs never affect another's.",
            ),
            Block::H2("Removing and cloning"),
            Block::P(
                "Removing a card, or moving it off the mainboard, clears its star. Cloning a deck carries its MVPs along; exported decklists leave stars out.",
            ),
            Block::Note("MVPs also headline your shared deck page, beside the command zone."),
        ],
    },
    Guide {
        slug: "share-your-deck",
        title: "Share a deck with a link",
        summary: "One link makes any deck public: featured cards, stats, and the full list in any browser. No account needed.",
        tags: &["Deck building"],
        related: &["deck-mvps", "deck-stats"],
        blocks: &[
            Block::Lead(
                "Any deck can go public with one link, viewable in a browser with no account or app.",
            ),
            Block::H2("Create the link"),
            Block::Steps(&[
                "Open the deck and tap the [[share]] in its dock.",
                "Tap Create link.",
                "The link copies to your clipboard.",
            ]),
            Block::Image {
                file: "share-your-deck/01-share-dialog.webp",
                alt: "The Share deck dialog before sharing, with its Create link button",
                caption: None,
            },
            Block::P(
                "The link opens at app.zwipe.net until you turn it off. The same dialog has Copy link.",
            ),
            Block::H2("What viewers see"),
            Block::P(
                "The page mirrors your deck view: name, format, and tags up top, your commander and starred MVPs as card images (tap one for the full art), and the same collapsible sections as the app: Budget, Tags, Distributions, Mana, and Draw odds.",
            ),
            Block::Image {
                file: "share-your-deck/02-shared-page-app.webp",
                alt: "A shared deck in a browser at app.zwipe.net: its commander featured, the View and Group by controls, the card list below",
                caption: None,
            },
            Block::P(
                "Below is the full card list with Group by and Show, the [[funnel]] in the dock to filter it, expandable rows, collapsible headers, and a section for the tokens your cards make.",
            ),
            Block::Note("The page shows the deck as you last saved it, every time it opens."),
            Block::H2("Stop sharing"),
            Block::P(
                "Same dialog, Stop sharing. The link stops working right away, and sharing again makes a new one.",
            ),
            Block::Image {
                file: "share-your-deck/03-stop-share-2.webp",
                alt: "The dialog once shared, offering Copy link and Stop sharing",
                caption: None,
            },
        ],
    },
    Guide {
        slug: "deck-stats",
        title: "Read your deck stats and charts",
        summary: "The deck view fills with live numbers: counts, prices, mana curve, color and type breakdowns, and draw odds.",
        tags: &["Deck stats"],
        related: &["budgeting", "land-targets", "import-export"],
        blocks: &[
            Block::Lead("A deck's stats update as you build. They count the mainboard only."),
            Block::H2("Profile, Budget & Tags"),
            Block::P(
                "Profile: name, format, command zone, Cards count, and Power level, the Commander bracket from Exhibition (1) to cEDH (5). Budget: price target, Total price, and Average card price, with a USD / EUR / TIX chip in its header. Tags: deck tags and Oracle tags. The land target lives in Mana.",
            ),
            Block::H2("Charts"),
            Block::P(
                "Distributions has the Types, Card roles and Oracle tags charts; Card roles uses the same roles as the card chips. Card roles, Oracle tags and Draw odds show five rows with a See more. Mana holds Lands against their target, Average mana value, the curve (nonland cards by cost, 0 to 6+), Colors, and mana fulfillment, with a ✔ per color your mana base covers.",
            ),
            Block::Image {
                file: "deck-stats/01-distributions.webp",
                alt: "The Distributions section: average power and toughness, type bars, and roles below",
                caption: None,
            },
            Block::H2("Draw odds"),
            Block::P(
                "The chance of drawing at least one card of a category (lands, or any other) by a chosen turn, stepped turn by turn.",
            ),
            Block::Image {
                file: "deck-stats/02-draw-odds.webp",
                alt: "The Draw odds section with the turn stepper on the opening hand",
                caption: None,
            },
            Block::H2("Warnings"),
            Block::P(
                "Rule problems (size, copy limit, off-color, missing commander, over budget, under land target) list here with a count badge and often a one-tap fix.",
            ),
        ],
    },
    Guide {
        slug: "import-export",
        title: "Import and export decklists",
        summary: "Bring a deck in from an Archidekt link or pasted text, and copy any deck back out as a plain decklist.",
        tags: &["Importing", "Deck building"],
        related: &["deck-stats"],
        blocks: &[
            Block::Lead("Zwipe reads and writes plain decklists."),
            Block::H2("Importing"),
            Block::P(
                "On Import, pick a source (Text, or an Archidekt URL), a Mode (add to or replace the board), and a Board (Main, Maybe, or Side). Text is one card per line, like \"1 Sol Ring.\"",
            ),
            Block::Image {
                file: "import-export/01-import-paste.webp",
                alt: "The Import screen with a decklist pasted in and the source, mode, and board chips set",
                caption: None,
            },
            Block::P(
                "After importing, matched cards show under Imported and anything unmatched under Failed, each with a reason.",
            ),
            Block::Image {
                file: "import-export/02-imported.webp",
                alt: "A finished import, matched cards listed under Imported with a count",
                caption: None,
            },
            Block::H2("Exporting"),
            Block::P(
                "Export builds a plain decklist with Commander, Deck, Maybeboard, and Sideboard sections. Toggle which boards to include, then tap the [[copy]].",
            ),
            Block::Image {
                file: "import-export/03-export.webp",
                alt: "The Export screen: board chips above a decklist split into Commander and Deck",
                caption: None,
            },
            Block::Note("The export pastes straight back into the importer."),
        ],
    },
    Guide {
        slug: "keyboard-shortcuts",
        title: "Keyboard shortcuts",
        summary: "On a computer, app.zwipe.net answers the keyboard: swipe, move between cards, and act on one without the mouse. Shift+H lists every key a page knows.",
        tags: &["Getting started"],
        related: &["swipe-to-build", "deck-card-list", "card-search"],
        blocks: &[
            Block::Lead(
                "On a computer, the web app at app.zwipe.net answers the keyboard. Shift+H lists every key the page you're on knows.",
            ),
            Block::Image {
                file: "keyboard-shortcuts/01-shortcuts-sheet.webp",
                alt: "The keyboard shortcuts sheet over a deck's cards, its keys grouped under Navigation, Card, and Page",
                caption: None,
            },
            Block::H2("On every page"),
            Block::Bullets(&[
                "`/` puts the cursor in the card search.",
                "`t` opens the themes, `?` the page's hint, and Shift+H this list.",
                "`Esc` closes the open sheet, or goes back.",
                "`gg` and Shift+G jump to the top or the bottom; `j` and `k` scroll.",
            ]),
            Block::H2("Add and Remove"),
            Block::Bullets(&[
                "The arrow keys swipe the top card, and so do `l` (right), `h` (left), `k` (up), and `j` (down). `u` undoes too.",
                "`f` opens the filter and `/` filters by name.",
                "`d` shows the card's details, `v` flips it, `p` opens its Printings, and Shift+P its Send to.",
                "`r` deals a fresh stack, and on Add `s` turns Synergy on or off.",
            ]),
            Block::H2("A deck's cards"),
            Block::Bullets(&[
                "The arrow keys, or `h`, `j`, `k`, and `l`, move between cards; Home and End jump to the first or last. Enter opens the card.",
                "On the card under the cursor: `p` Printings, Shift+P Send to, `i` its image, `v` flip, `d` details.",
                "On your own deck: `m` moves it to the maybeboard, `s` to the sideboard, Shift+S stars it, Shift+F flags it, and `-` and `+` change the count.",
                "`u` or Ctrl+Z undoes, `f` filters, `/` filters by name, `q` opens Quick add, and `a`, `r`, and `e` open Add, Remove, and Edit deck.",
            ]),
            Block::H2("Search results"),
            Block::Bullets(&[
                "The arrow keys, or `h`, `j`, `k`, and `l`, move between the cards as they're laid out; Home and End jump to the first or last. Enter opens the card.",
                "On the card under the cursor: `p` Printings, Shift+P Send to, `i` its image, `v` flip, `d` details.",
                "From the search box, `↓` or `Esc` puts the cursor back on the cards.",
            ]),
            Block::H2("A card page"),
            Block::Bullets(&[
                "`p` opens Printings and Shift+P opens Send to.",
                "In Printings, `←` and `→`, or `h` and `l`, step through the printings, `v` flips one, and Enter picks it.",
            ]),
            Block::Note("The app on a phone and zwipe.net don't take these keys."),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::site::WEB_BASE;

    #[test]
    fn a_screenshot_is_served_once_by_zwipe_net() {
        assert_eq!(
            image_url("getting-started/01-create-deck.webp"),
            format!("{WEB_BASE}/guide-images/getting-started/01-create-deck.webp")
        );
    }

    #[test]
    fn the_web_s_guides_say_what_the_commander_deal_does() {
        for guide in GUIDES {
            for block in guide.blocks_on(Surface::Web) {
                if let Block::P(text) | Block::Note(text) = block {
                    assert!(
                        !text.contains("100 deep") && !text.contains("one color identity"),
                        "{}: {text}",
                        guide.slug
                    );
                }
            }
        }
    }

    #[test]
    fn every_guide_has_a_screenshot() {
        let bare: Vec<&str> = GUIDES
            .iter()
            .filter(|guide| screenshots(guide).is_empty())
            .map(|guide| guide.slug)
            .collect();
        assert!(bare.is_empty(), "guides without a screenshot: {bare:?}");
    }

    #[test]
    fn every_related_guide_exists_and_slugs_are_unique() {
        for guide in GUIDES {
            for related in guide.related {
                assert!(
                    find(related).is_some(),
                    "{}: no guide {related}",
                    guide.slug
                );
            }
            assert_eq!(
                GUIDES.iter().filter(|g| g.slug == guide.slug).count(),
                1,
                "{}",
                guide.slug
            );
            assert!(!guide.tags.is_empty(), "{} has a tag", guide.slug);
        }
    }

    #[test]
    fn a_guide_that_points_to_guides_below_lists_them() {
        for guide in GUIDES {
            let points_below = guide.blocks.iter().any(|block| {
                matches!(block, Block::Lead(text) | Block::P(text) | Block::Note(text) if text.contains("guide below"))
            });
            assert!(
                !points_below || !guide.related.is_empty(),
                "{} says a guide is below but lists none",
                guide.slug
            );
        }
        let started = find("getting-started").map(|g| g.related);
        assert_eq!(
            started,
            Some(
                &[
                    "deck-stats",
                    "remove-cards",
                    "budgeting",
                    "land-targets",
                    "import-export"
                ][..]
            ),
            "the deck view's stats, then each tool its paragraph names, in its order"
        );
    }
}
