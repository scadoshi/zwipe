//! The guides zwipe.net and the web app both draw: each one's text, figures and screenshots as
//! data, so the two sites render one copy. Keep articles accurate to the app: labels here mirror
//! the real UI text, `backticks` mark a keyword and `[[name]]` draws an icon button as its icon
//! (a name zwipe-ui's hint `KeyIcon` knows), so no line describes an icon in words. Never
//! describe where synergy data comes from, only what it does for the user.
//!
//! The screenshots are files served once, by zwipe.net ([`image_url`]); [`image_size`] gives each
//! one's size so a page holds its place while it loads.

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

/// The files `guide`'s screenshots name, in order.
#[must_use]
pub fn screenshots(guide: &Guide) -> Vec<&'static str> {
    guide
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Image { file, .. } => Some(*file),
            _ => None,
        })
        .collect()
}

/// The guide whose slug is `slug`.
#[must_use]
pub fn find(slug: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|guide| guide.slug == slug)
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
        related: &[],
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
            Block::P("Open the deck, tap Cards, then Add, and act on the top card with a flick:"),
            Block::Swipe(&[
                ("right", "add the card to your deck"),
                ("left", "skip it"),
                ("up", "send it to your maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::Image {
                file: "getting-started/02-add-cards.webp",
                alt: "The Add deck cards screen: a Goblin on top of the stack, the From row's Search chip, the bolt lit in the dock",
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
                file: "getting-started/03-deck-cards.webp",
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
            Block::Lead(
                "The Add deck cards screen deals cards as a stack, and you decide each with a flick.",
            ),
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
                "The [[funnel]] decides which cards are dealt; a dot on it marks an active filter. Sorting only reorders. The Maybeboard chip after From deals your maybeboard instead; a right swipe there moves a card to the mainboard. Each source keeps its own filter.",
            ),
            Block::Image {
                file: "swipe-to-build/02-filter-sheet.webp",
                alt: "The filter sheet with Types picked and a mana value range set",
                caption: None,
            },
            Block::Image {
                file: "swipe-to-build/03-maybeboard-stack.webp",
                alt: "The From row switched to Maybeboard, its chip lit, dealing saved cards",
                caption: None,
            },
            Block::H2("Undo"),
            Block::P(
                "Swipe down to undo: an added card comes back out, a skipped card comes back. Your place and undo history stay with each deck while the app is open. The [[refresh]], or a filter change, deals a fresh stack and clears the history.",
            ),
            Block::Note(
                "At the end of the stack, swipe down on the empty card to step back through your last swipes.",
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
                "The Boards row picks the pool: Main, Maybe, Side, or All. The [[funnel]] narrows it. The [[refresh]] restarts the stack and clears your undo history.",
            ),
            Block::Image {
                file: "remove-cards/01-remove-swipe.webp",
                alt: "The Remove deck cards screen mid-swipe, with Boards set to Main",
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
                "`Mana`: mana value, color identity (`exact` or `within` a set of colors), and colors produced.",
                "`Combat`: power and toughness, `exact` or a `range`.",
                "`Rarity`, `Set`, `Artist`: include or exclude values.",
                "`Card roles`: 27 roles like `Ramp`, `Removal`, `Draw`, `Wipe`.",
                "`Oracle tags`: thousands of functional tags (spot removal, ramp, reanimation); include or exclude, with a curated set up front and search for the rest.",
                "`Format`: `Is commander in` and `Is legal in`.",
                "`Price`: min and max in `USD`, `EUR`, or `TIX`.",
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
                "The same value in both `include` and `exclude` stops the [[check]] with \"Filter can't both include and exclude\"; the sheet stays open so you can fix it.",
            ),
            Block::Image {
                file: "filtering/02-filter-hint.webp",
                alt: "The Filters hint: stack sections and apply with the check in the dock",
                caption: None,
            },
            Block::H2("Sorting"),
            Block::P(
                "Sort by `Name`, `Mana value`, `Power`, `Toughness`, `Rarity`, `Release Date`, `Price`, `Popularity`, or `Random`. Sorting only reorders. Each screen keeps its own filter and sort for the session.",
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
        title: "Group, filter, and browse your lists",
        summary: "The deck list and card lists use the same controls: Group by sorts into sections, Show narrows the list, and headers collapse.",
        tags: &["Deck building", "Cards"],
        related: &["filtering", "deck-tags", "swipe-to-build"],
        blocks: &[
            Block::Lead("Your deck list and each deck's card list share the same controls."),
            Block::H2("The deck list"),
            Block::P(
                "Group by sections your decks by Format, Color, or Tag; a deck with several tags appears under each. None is one collapsible group headed All. Color makes a group per exact color combination, headed by its mana pips. Show narrows the list: color pips keep decks whose identity contains every pip you pick, tag chips match any tag, and All resets.",
            ),
            Block::Image {
                file: "organize-and-browse/01-deck-list-by-tag.webp",
                alt: "The deck list grouped by Tag, with the Show pips and tag chips above",
                caption: None,
            },
            Block::H2("A deck's card list"),
            Block::P(
                "Group by sections cards by Type, Mana value, Color, or Card role; None is one list. Boards is three independent toggles, Main, Maybe, and Side; turn the last one off and Main snaps back on. Show toggles Tokens, the Command zone, and Art.",
            ),
            Block::H2("Collapse what you're done with"),
            Block::P(
                "Tap a group header on either list to fold it. On the card list, Tokens, Lands, Maybeboard, and Sideboard fold too. Collapses last for the visit.",
            ),
            Block::Image {
                file: "organize-and-browse/02-cards-by-type-collapsed.webp",
                alt: "A card list grouped by Type with the Creatures section collapsed",
                caption: None,
            },
            Block::H2("Quick add"),
            Block::P(
                "Quick add at the top of the card list searches by name and adds a result to your mainboard with one tap. It reaches cards you skipped while swiping.",
            ),
            Block::Image {
                file: "organize-and-browse/03-quick-add-results.webp",
                alt: "Quick add results floating over the card list",
                caption: None,
            },
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
                "Synergy is the [[bolt]] in the Add deck cards dock, there when the source is Search and the deck has a commander. On, the stack starts with fitting cards, best first, 25 at a time, shuffled within each batch per deck per day. Off, you browse everything legal. It's on by default, and toggling it reloads the stack.",
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
        related: &["commander-and-formats", "swipe-to-build"],
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
            Block::H2("The Commander maybeboard screen"),
            Block::P(
                "Open Commander maybeboard from the menu. Saves show newest first as card rows with art (the Art chip hides it). Expand a row for details and four actions:",
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
                "Quick add searches commanders by name; tap a result to save it. The color pips and the [[funnel]] narrow the list of saves.",
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
    },
    Guide {
        slug: "playstyle",
        title: "Playstyle",
        summary: "What your decks say about how you play, and commanders ranked against them, with the reasons each one made the list.",
        tags: &["Commander"],
        related: &["commander-maybeboard", "synergy", "activity"],
        blocks: &[
            Block::Lead(
                "Playstyle reads every deck you have, in any format, and ranks commanders you haven't built yet.",
            ),
            Block::H2("Finding it"),
            Block::Bullets(&[
                "On the Home screen, tap Playstyle on the Colors panel. It's also in the menu.",
                "Your colors, card roles and Oracle tags come first.",
                "Under them, Suggested commanders lists your top 10 as card rows, best fit first. Its `?` explains the ranking.",
                "Tap a color chip in the Show row to narrow to those colors; tap again to clear.",
                "Staples, at the bottom, lists the cards in the most of your decks: each row shows the share of decks that run it, and an expanded row the count, like 4 of 6. Basic lands are left out. Type and Color chips narrow it, and it has a `?` too.",
            ]),
            Block::H2("What your decks say"),
            Block::P(
                "Colors opens first: a bar per color with its mana pip and how many of your cards are that color, colorless last. It's the same panel as on Home. Tap a section under it to open it.",
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
                "Each has a `?` in its header.",
            ]),
            Block::H2("How a tag's impact adds up"),
            Block::Bullets(&[
                "Decks is how many of your decks have the tag, Cards how many cards carry it across them. More of either, more impact.",
                "Impact is how much more your decks lean on a tag than Magic cards do overall. Your top tag scores 100.",
                "A tag your decks use no more than Magic overall doesn't show up.",
                "Each column has its own `?`, too.",
            ]),
            Block::Image {
                file: "playstyle/02-oracle-tag-impact.webp",
                alt: "Oracle tags: each tag's Decks, Cards and Impact, the top tag at 100",
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
                "Create deck starts a new deck with it as your commander.",
                "Maybeboard saves it for later.",
                "Remove hides it for a week and pulls the next one up.",
            ]),
            Block::H2("Swipe deeper"),
            Block::P(
                "Swipe deals the same ranking as a stack, up to 100 deep. Tap the [[funnel]] while you swipe to narrow it to one color identity.",
            ),
            Block::Swipe(&[
                ("right", "start a new deck with this commander"),
                ("left", "skip it for a week"),
                ("up", "save it to your commander maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::P(
                "Skipped commanders come back after seven days, or sooner with `Clear` beside Clear skips in the Danger zone panel.",
            ),
            Block::Note(
                "You need a deck with some cards in it before anything shows up. Any format counts.",
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
                file: "activity/01-heatmap-and-totals.webp",
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
                file: "activity/02-card-roles.webp",
                alt: "Card roles with Added picked in the Show row: each role's share of the cards dealt to you",
                caption: None,
            },
            Block::Image {
                file: "activity/03-most-skipped.webp",
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
                "The Land target stepper starts at \"Not set.\" The first tap sets a default (about 37 for 100-card formats, 17 for 60-card), then you adjust.",
            ),
            Block::H2("Feedback"),
            Block::P(
                "Cross the target while editing and Zwipe toasts \"Land target reached\" or \"Below land target,\" and a standing warning shows whenever you're under.",
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
                "The Oracle tags field shows a curated set of the most useful tags up front, and search reaches the rest, about 4,500 in all. A deck can hold up to 30.",
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
                "The Oracle tags dictionary describes every Oracle tag, about 4,500 in all.",
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
                file: "card-roles/01-role-chips.webp",
                alt: "An expanded card row with its Card roles chips, one opened to its Oracle tags",
                caption: None,
            },
            Block::H2("Roles"),
            Block::P(
                "The deck view's Distributions section has a Roles chart, the same roles counted across the whole deck. Tap a role to see the Oracle tags your cards carry under it.",
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
                file: "tags-roles-and-oracle-tags/02-role-to-oracle-tags.webp",
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
                file: "deck-mvps/01-featured-strip.webp",
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
                "Open the deck and tap the [[share]] beside its profile.",
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
                "Below is the full card list with Group by, Show, and Filter by, expandable rows, collapsible headers, and a section for the tokens your cards make.",
            ),
            Block::Note("The page shows the deck as you last saved it, every time it opens."),
            Block::H2("Stop sharing"),
            Block::P(
                "Same dialog, Stop sharing. The link stops working right away, and sharing again makes a new one.",
            ),
            Block::Image {
                file: "share-your-deck/03-stop-share.webp",
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
                "Distributions has the Types, Roles and Oracle tags charts; Roles uses the same roles as the card chips. Roles, Oracle tags and Draw odds show five rows with a See more. Mana holds Lands against their target, Average mana value, the curve (nonland cards by cost, 0 to 6+), Colors, and mana fulfillment, with a ✔ per color your mana base covers.",
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
                "After importing, matched cards show under Imported and anything unmatched under Unresolved, each with a reason.",
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
}
