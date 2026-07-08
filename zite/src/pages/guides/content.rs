//! Guide article content, as data. Each `Guide` renders into the privacy-page
//! long-form layout via `super::render_block`. Keep articles accurate to the
//! app: labels here mirror the real in-app UI text. Never describe where
//! synergy data comes from; only its user-facing behavior.

/// A block of article content. Rendered in order by `super::render_block`.
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
}

pub struct Guide {
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub category: &'static str,
    pub blocks: &'static [Block],
}

pub static GUIDES: &[Guide] = &[
    // Start
    Guide {
        slug: "getting-started",
        title: "Getting started with Zwipe",
        summary: "Zwipe builds Magic: The Gathering decks with swipes instead of forms. Here's the whole model.",
        category: "Start",
        blocks: &[
            Block::Lead(
                "Zwipe builds Magic: The Gathering decks by swiping through cards one at a time, instead of typing names into a form.",
            ),
            Block::H2("Create a deck"),
            Block::P(
                "Make a deck, name it, and pick a format. A commander format also unlocks the command-zone fields: commander, and where relevant partner, background, or signature spell.",
            ),
            Block::H2("Build by swiping"),
            Block::P("Open the deck, tap Add Deck Cards, and act on the top card with a flick:"),
            Block::Swipe(&[
                ("right", "add the card to your deck"),
                ("left", "skip it"),
                ("up", "send it to your maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::P(
                "Every add saves instantly and syncs across your devices. Skips stick too: swipe left and that card stays out of this deck, even after you close the app. Each deck also keeps your place, so you can hop between decks and pick up where you left off.",
            ),
            Block::H2("Shape the stack"),
            Block::P(
                "On a commander deck, the Synergy toggle keeps the stack to cards that fit your commander. Filter narrows by color, type, keyword, price, and more. Sorting reorders what you see without changing which cards are eligible.",
            ),
            Block::H2("Review and refine"),
            Block::P(
                "As the deck grows, the deck view fills with live stats, charts, and a warnings list with one-tap fixes. Trim with the remove flow, set a budget or land target, and import or export any time. Each area has its own guide below.",
            ),
        ],
    },
    // Build
    Guide {
        slug: "swipe-to-build",
        title: "Build a deck by swiping",
        summary: "The add-cards flow: right to add, left to skip, up to maybeboard, down to undo, with synergy and filters to shape the stack.",
        category: "Build",
        blocks: &[
            Block::Lead(
                "The Add Deck Cards screen deals cards as a stack, and you decide each with a flick.",
            ),
            Block::H2("The four swipes"),
            Block::P(
                "A first-visit hint (\"Swipe to build\") covers the gestures, and the \"?\" in the header reopens it.",
            ),
            Block::Swipe(&[
                ("right", "add the card to your deck"),
                ("left", "skip it"),
                ("up", "send it to your maybeboard"),
                ("down", "undo your last swipe"),
            ]),
            Block::P(
                "Only the top card is interactive, and double-faced cards flip to show the back.",
            ),
            Block::H2("Skips are remembered"),
            Block::P(
                "A left swipe keeps that card out of this deck for good, even after you reopen the app. Clear skips in the deck's More menu brings them back. The swipe-memory guide covers this in full.",
            ),
            Block::H2("Synergy"),
            Block::P(
                "On a commander deck, the Synergy chip keeps the stack to cards that fit your commander, best fits first. Off, you browse every legal card. It's on by default and re-deals the stack when toggled.",
            ),
            Block::Note(
                "If synergy is still warming up, Zwipe shows the full pool and tells you once.",
            ),
            Block::H2("Filter, sort, and the maybeboard"),
            Block::P(
                "Filter decides which cards are dealt; a dot marks an active filter. Sorting reorders the current set only. The \"From\" row swaps the stack to your Maybeboard, where a right swipe promotes a card into the deck.",
            ),
            Block::H2("Undo"),
            Block::P(
                "Swipe down to undo your last swipe; undoing an add removes the card again, and undoing a skip brings the card back. Your place and undo history travel with each deck, so leaving and returning picks up where you left off. Refresh (or changing the filter) starts a fresh stack and clears the history.",
            ),
            Block::Note(
                "At the very end of the stack, swipe down on the empty card to step back through your last swipes.",
            ),
        ],
    },
    Guide {
        slug: "remove-cards",
        title: "Remove cards from a deck",
        summary: "The remove flow mirrors building: swipe through the cards already in your deck and trim.",
        category: "Build",
        blocks: &[
            Block::Lead(
                "The remove flow is the mirror of building: instead of the whole card pool, it deals only the cards already in your deck.",
            ),
            Block::H2("The four swipes"),
            Block::P("A first-visit hint (\"Swipe to trim\") covers them:"),
            Block::Swipe(&[
                ("right", "remove the card from your deck"),
                ("left", "keep it"),
                (
                    "up",
                    "move it between boards (to the maybeboard, or a maybeboard card up to main)",
                ),
                ("down", "undo your last swipe"),
            ]),
            Block::H2("Boards, filter, and refresh"),
            Block::P(
                "The Boards row picks the pool you're trimming: Main, Maybe, Side, or All. Filter narrows it further, and Refresh restarts the stack and clears your undo history.",
            ),
            Block::Note(
                "A card you remove also stops showing up when adding to this deck, the same as a skip. Clear the deck's skips to bring removed cards back into the pool.",
            ),
        ],
    },
    Guide {
        slug: "swipe-memory",
        title: "Swipe memory: skips that stick",
        summary: "Skips and removals are remembered per deck, survive closing the app, and every deck keeps your place. Clear skips to start over.",
        category: "Build",
        blocks: &[
            Block::Lead(
                "Zwipe remembers the cards you pass on. Skip a card and it stays out of that deck's suggestions, so you never have to swipe past the same card twice.",
            ),
            Block::H2("Skips stay skipped"),
            Block::P(
                "A left swipe on the add screen tells Zwipe you don't want that card in this deck. It stops appearing while you build, and it stays gone after you close and reopen the app. Removing a card from a deck does the same, so trimmed cards don't come back as suggestions.",
            ),
            Block::H2("It's per deck"),
            Block::P(
                "Memory is scoped to the deck you're building. Skipping a card in one deck never hides it in another, so each deck's suggestions reflect only your choices for that deck.",
            ),
            Block::H2("Pick up where you left off"),
            Block::P(
                "Every deck keeps its place in the stack, along with your recent swipes for undo. Jump out to tweak another deck and come back, and Zwipe drops you right back where you were instead of starting over.",
            ),
            Block::H2("Changed your mind? Clear skips"),
            Block::P(
                "Open the deck's More menu and tap Clear skips. Every card you skipped or removed for that deck comes back into the pool, and swiping starts fresh.",
            ),
            Block::Note("Clearing skips can't be undone, and it only affects the one deck."),
        ],
    },
    // Cards
    Guide {
        slug: "filtering",
        title: "Filter the card pool",
        summary: "Stack filters to control which cards you see: name, color, mana, type, text, keyword, rarity, set, artist, price, category, and format.",
        category: "Cards",
        blocks: &[
            Block::Lead(
                "Filters decide which cards you see, whether you're adding, browsing, or trimming. The sheet is an accordion, one section per attribute.",
            ),
            Block::H2("What you can filter on"),
            Block::Bullets(&[
                "`Name`: contains, or doesn't contain.",
                "`Oracle text`: rules text contains or excludes; match or exclude specific oracle words or keyword abilities.",
                "`Types`: the basic card types, or any type-line word like `legendary` or `dragon`.",
                "`Mana`: mana value, color identity (`exact` or `within` a set of colors), and colors produced.",
                "`Combat`: power and toughness, `exact` or a `range`.",
                "`Rarity`, `Set`, `Artist`: include or exclude values.",
                "`Flavor text`: contains, or doesn't contain.",
                "`Category`: 24 strategic roles (`Ramp`, `Removal`, `Draw`, `Board Wipe`, and so on).",
                "`Format`: `Is commander in` and `Is legal in`.",
                "`Price`: a currency (`USD`, `EUR`, or `TIX`) with optional min and max.",
            ]),
            Block::H2("Combining"),
            Block::P(
                "Sections combine with `AND`, so stack as many as you like. A multi-value section's `Any`/`All` toggle sets whether a card needs one selected value or all of them, and `include` and `exclude` can run at once.",
            ),
            Block::Note(
                "Set a value as both `include` and `exclude` and Zwipe reverts it, since nothing would match.",
            ),
            Block::H2("Sorting"),
            Block::P(
                "Sort by `Name`, `Mana value`, `Power`, `Toughness`, `Rarity`, `Release Date`, `Price`, `Popularity`, or `Random`. Sorting only reorders the set you're looking at; it never changes which cards are eligible. Your filter and sort follow you between screens.",
            ),
            Block::H2("Defaults and Reset"),
            Block::P(
                "Every screen starts from a default view, and that default isn't always empty. `Reset` returns you to it:",
            ),
            Block::Bullets(&[
                "Add cards on a commander deck: your commander's colors, the format's legal cards, in synergy order. Once you hit your land target, lands drop out too.",
                "Add cards on other formats: the format's legal cards, in default order.",
                "Pick a commander: every eligible commander, in popularity order.",
                "Remove and browse: an empty filter, over your deck's own cards.",
            ]),
            Block::P(
                "The dot on the `Filter` button means you've changed something from that default, a filter or a different sort. `Reset` undoes the change and drops you back to the default. Synergy is a separate mode, so it stays however you set it.",
            ),
            Block::Note(
                "These built-in defaults never light the dot, since you didn't set them: your commander's colors, format legality, the popularity or synergy order, and lands leaving once you're at target.",
            ),
            Block::H2("Categories"),
            Block::P(
                "A category is a card's role, and cards can hold several (`Lightning Bolt` is `Burn` and `Removal`). Zwipe assigns them automatically, so you just pick the roles you want.",
            ),
        ],
    },
    Guide {
        slug: "synergy",
        title: "Synergy-ranked cards",
        summary: "On a commander deck, Synergy keeps the stack to cards that work with your commander and shows the best fits first.",
        category: "Cards",
        blocks: &[
            Block::Lead(
                "On a commander deck, Synergy leads with the cards that actually work with your commander instead of every legal card.",
            ),
            Block::H2("Where and how"),
            Block::P(
                "The Synergy chip appears on the Add Deck Cards screen once your deck has a commander. On, the stack is limited to fitting cards, best first; off, you browse everything legal. It's on by default and re-deals when toggled.",
            ),
            Block::Note(
                "If a commander's synergy isn't ready yet, Zwipe shows all cards and says \"Synergy warming up\" once.",
            ),
            Block::H2("Synergy vs. sorting"),
            Block::P(
                "Synergy sets which cards appear and their order; Sort only reorders the current set. Keep Synergy on to stay on-theme, then sort within those picks.",
            ),
        ],
    },
    // Decks
    Guide {
        slug: "commander-and-formats",
        title: "Choose a commander & format",
        summary: "Pick a format, then a commander. Zwipe enforces each format's rules and your commander's color identity.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Your format sets the rules, your commander sets the colors, and Zwipe keeps the build inside both.",
            ),
            Block::H2("Pick a format"),
            Block::P(
                "The Format picker is single-select: tap a format to see its details (pool, size, life, command zone), and tap again to clear. Commander formats are listed first.",
            ),
            Block::Bullets(&[
                "Commander formats: `Brawl`, `Commander`, `Duel Commander`, `Historic Brawl`, `Oathbreaker`, `Pauper Commander`, `PreDH`, `Standard Brawl`.",
                "Other formats: `Alchemy`, `Explorer`, `Future`, `Gladiator`, `Historic`, `Legacy`, `Modern`, `Old School`, `Pauper`, `Penny Dreadful`, `Pioneer`, `Premodern`, `Standard`, `Timeless`, `Vintage`.",
            ]),
            Block::P(
                "Each format has its own rules: Commander is 100 cards, singleton, 40 life; Oathbreaker is 60 with a planeswalker and signature spell; 60-card constructed formats allow four copies. Changing the format clears your commander.",
            ),
            Block::H2("Pick a commander"),
            Block::P(
                "Each command-zone field (Commander, plus Partner, Background, or Signature spell when relevant) offers two paths:",
            ),
            Block::Steps(&[
                "Type to search. Filter on limits results to eligible cards; off searches any card by name.",
                "Tap Zwipe for the swipe picker, most-played first: right to choose, left to skip, down to undo.",
            ]),
            Block::P(
                "Partner and Background appear only when your commander supports them. Oathbreaker needs a signature spell within the planeswalker's colors.",
            ),
            Block::H2("Color identity"),
            Block::P(
                "In commander formats, your deck's colors come from the commander (plus partner and background). The add screen stays inside those colors, and anything off-color is flagged in warnings.",
            ),
        ],
    },
    Guide {
        slug: "budgeting",
        title: "Set a budget with price targets",
        summary: "Give a deck a price target and Zwipe tracks the running total, warns as you approach it, and offers buy links.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Give a deck a price target and Zwipe tracks the running total as you build.",
            ),
            Block::H2("Set a budget"),
            Block::P(
                "On the edit form, pick a currency (USD, EUR, or TIX) and an amount under Price target; leave it blank for no budget. It shows in the deck's Profile.",
            ),
            Block::H2("As it fills up"),
            Block::P(
                "Zwipe alerts you once as the deck crosses 50%, 75%, and 100% of the budget (for example, \"Deck at 82.50% of your $50.00 budget\"), and a standing warning appears if you go over.",
            ),
            Block::Note(
                "Cards with no price in your currency count as zero, so the total is a floor, not an appraisal.",
            ),
            Block::H2("Prices and buying"),
            Block::P(
                "The Stats section shows Total price and Average card price, switchable across USD, EUR, and TIX, and each card lists its prices while you swipe. Buy deck (under More) opens mass-entry links to TCGplayer and Card Kingdom.",
            ),
        ],
    },
    Guide {
        slug: "land-targets",
        title: "Set a land target",
        summary: "Tell Zwipe how many lands you want and it warns you as the deck fills, so you don't finish short on mana.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Set the number of lands you want and Zwipe warns you as the deck fills, so you don't finish short on mana.",
            ),
            Block::H2("Set the target"),
            Block::P(
                "The Land target stepper starts at \"Not set.\" The first tap seeds a sensible default (about 37 for 100-card formats, 17 for 60-card), then you adjust.",
            ),
            Block::H2("Feedback"),
            Block::P(
                "Cross the target while editing and Zwipe toasts \"Land target reached\" or \"Below land target,\" and a standing warning shows whenever you're under.",
            ),
            Block::H2("Lands leave the stack"),
            Block::P(
                "When you open the add screen with your land count already at target (your target, or the format default), Zwipe excludes lands from the swipe stack so it stops dealing you ones you don't need. It adds `Land` to the `Basic types` exclude filter. To bring lands back while you're still at target, remove that exclude yourself; a `Reset` keeps it in place, since excluding lands is the default once you're at target. Reaching the target mid-session takes effect the next time you enter the screen.",
            ),
            Block::Note(
                "Even with no target set, warnings use your format's default land count as a fallback.",
            ),
        ],
    },
    Guide {
        slug: "deck-tags",
        title: "Tag decks by archetype",
        summary: "Label a deck's game plan with up to five strategy tags: Aggro, Tokens, Reanimator, and over a hundred more.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Tags label a deck's game plan: Aggro, Tokens, Reanimator, Stax, and over a hundred more.",
            ),
            Block::H2("Adding tags"),
            Block::P(
                "Open Tags from the edit form, then tap to add or remove; each tag shows its definition up top. Search by name, and use × to clear them all.",
            ),
            Block::H2("How many"),
            Block::P(
                "Up to five tags per deck (shown as N/5), from one flat alphabetical list. Tags are only ever added, never renamed, so old ones keep working.",
            ),
        ],
    },
    Guide {
        slug: "deck-stats",
        title: "Read your deck stats & charts",
        summary: "The deck view fills with live numbers and charts: counts, prices, mana curve, color and type breakdowns, fulfillment, and draw odds.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Open a deck and its stats build themselves. Everything reflects your mainboard; sideboard and maybeboard cards are excluded.",
            ),
            Block::H2("Profile"),
            Block::P(
                "Summarizes the deck: name, format, commander (and partner, background, or signature spell), tags, land and price targets, and a Power level, the Commander bracket from Exhibition (1) to cEDH (5).",
            ),
            Block::H2("Stats"),
            Block::P(
                "Cards, Average mana value, Lands, Total price, and Average card price, with a USD / EUR / TIX currency chip.",
            ),
            Block::H2("Charts"),
            Block::P(
                "Distributions cover card type, strategic category, and color. Mana holds the mana curve (nonland cards by cost, 0 to 6+) and mana fulfillment (whether your mana base makes enough of each color, with a ✔ when covered).",
            ),
            Block::H2("Draw odds"),
            Block::P(
                "The chance of drawing at least one card of a type (lands, or any category) by a chosen turn, updated live and stepped turn by turn.",
            ),
            Block::H2("Warnings"),
            Block::P(
                "Rule problems (size, copy limit, off-color, missing commander, over budget, under land target) list here with a count badge and often a one-tap fix.",
            ),
        ],
    },
    Guide {
        slug: "import-export",
        title: "Import & export decklists",
        summary: "Bring a deck in from an Archidekt link or pasted text, and copy any deck back out as a plain decklist.",
        category: "Decks",
        blocks: &[
            Block::Lead(
                "Zwipe reads and writes plain decklists, so you can move decks in and out.",
            ),
            Block::H2("Importing"),
            Block::P(
                "On Import, pick a source (Text, or an Archidekt URL), a Mode (add to or replace the board), and a Board (Main, Maybe, or Side). Text is one card per line, like \"1 Sol Ring.\"",
            ),
            Block::P(
                "After importing, matched cards show under Imported and anything unmatched under Unresolved, each with a reason.",
            ),
            Block::H2("Exporting"),
            Block::P(
                "Export builds a plain decklist with Commander, Deck, Maybeboard, and Sideboard sections. Toggle which boards to include, then tap Copy.",
            ),
            Block::Note(
                "The export matches the importer, so a copied list pastes straight back in, or over to a friend.",
            ),
        ],
    },
];
