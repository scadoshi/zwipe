//! Shared changelog data.
//!
//! The release history, compiled into every surface: the website (`zite`) and
//! the app (`zwiper`) render it, and the server (`zerver`) serves it at
//! `/api/changelog` so new clients can fetch fresh entries without an app
//! resubmit. This is the single source of truth; the wire types in
//! [`crate::http::contracts::changelog`] project it into an owned, serializable
//! response.
//!
//! Data lives here (pure, no UI deps) rather than in `zwipe-components` so the
//! server can serve it without depending on a UI crate.

/// One shipped version and its notes. Notes sourced from the App Store "What's
/// New" history, dates from the App Store Connect submission history (public
/// "Ready for Distribution" date); newest first. Em dashes recast for copy
/// style.
pub struct Release {
    /// Semantic version string, e.g. `"1.6.0"`.
    pub version: &'static str,
    /// Human-readable release date, e.g. `"Jul 12, 2026"`.
    pub date: &'static str,
    /// User-facing release notes, one bullet per entry.
    pub entries: &'static [&'static str],
}

/// Versions in progress for the next release. Rendered at the top of the
/// changelog with an "Upcoming" badge instead of "Latest".
pub const UPCOMING: &[Release] = &[Release {
    version: "1.7.2",
    date: "Coming soon",
    entries: &[
        "Fixed the deck's Tags section so long oracle and deck tags no longer overlap their labels.",
        "The deck cards grouping once labeled Category now reads Card role, matching the filter and the rest of the app.",
    ],
}];

/// Shipped releases, newest first.
pub const RELEASES: &[Release] = &[
    Release {
        version: "1.7.1",
        date: "Jul 17, 2026",
        entries: &[
            "Cards without a printed image now show as a clean text card with type, mana cost, rules, and power or toughness, so nothing is hidden while you swipe.",
            "See real example cards for any oracle tag: open the dictionary and tap Examples to swipe through cards that use it, most iconic first.",
            "Use a tag straight from the dictionary: it drops into your deck's strategy or your card filter without losing your place, and the dictionary now opens from the filter too.",
            "Every oracle tag now has a plain-language description, covering the full catalog, with an accuracy pass across thousands of them.",
            "Tap an oracle tag on one of your cards to read what it does right there, and open its example cards without leaving the card.",
            "Card details restyled to match the rest of the app, with the mana cost by the title and color-coded type, rarity, and set, and it now opens scrolled to the top.",
            "The back gesture now closes an open dictionary, picker, or filter one layer at a time, instead of leaving the whole screen.",
            "Cleaner scrolling throughout, with hidden scrollbars and soft fade edges on lists and dialogs.",
        ],
    },
    Release {
        version: "1.7.0",
        date: "Jul 14, 2026",
        entries: &[
            "New oracle-tag dictionary: browse every tag by letter or search names and descriptions, opened from the oracle-tag picker.",
            "Sharper oracle tags: better tags under each card role, closer archetype matches, and over a thousand plain-language descriptions.",
            "Filters and pickers now open instantly, loaded once in the background instead of refetching each time.",
            "Flip double-faced cards from the card details, with the Flip button now in the footer and everything in one scroll.",
            "The Export screen now shows a loading placeholder instead of a spinner.",
            "The changelog now updates on its own, without waiting for an app update.",
        ],
    },
    Release {
        version: "1.6.0",
        date: "Jul 12, 2026",
        entries: &[
            "In-app changelog, from your Profile.",
            "Cleaner deck card screen: squircle mana pips, an inline price tag, and power/toughness on each row.",
            "Theme and dark-mode controls moved to your Profile.",
            "Seventeen new themes (thirty-one total), including VS Code, GitHub, One Dark, Kanagawa, Ayu, Night Owl, docs.rs, Synthwave '84, Vantablack, PowerShell, and Hackerman.",
            "A new Achromatopsia theme for total color blindness, plus a contrast pass on every theme so dimmed text stays readable.",
            "Your theme now persists across the app and website, including the sign-in screens and between visits.",
            "Buy a card from the home screen: tap its price for TCGplayer or Card Kingdom.",
            "Fixed importing double-faced cards like Boggart Trawler // Boggart Bog.",
            "Light-mode polish across every theme.",
            "Oracle tags: community-maintained tags for what cards do (removal, ramp, card advantage, tutors, and more).",
            "Card roles on every card: expand a card to see its roles (removal, ramp, card advantage, aggression), and tap one for the oracle tags underneath.",
            "Choose oracle tags for your deck's strategy, or pick an archetype like Aggro or Aristocrats to seed matching ones.",
            "New in-app guides for deck tags, card roles, and oracle tags.",
            "Deck color identity now shows as mana pips next to each deck's name.",
            "Every mana pip now tints its outline and glyph to its own color in any theme.",
            "Reorganized deck view: Profile, Budget, and Tags as collapsible sections, with the price target as a goal-versus-actual row.",
            "Deck lands moved to the Mana section, shown against your land target.",
            "Shared deck pages now show card roles and the full deck price, command zone included.",
            "More polish: centered hybrid mana symbols, clearer section labels, and smoother card-detail reveals.",
        ],
    },
    Release {
        version: "1.5.0",
        date: "Jul 10, 2026",
        entries: &[
            "Swipe back: drag from the left edge to return to the previous page.",
            "Filters that remember: every screen keeps its own filters per deck.",
            "An expanded card now stays highlighted so you keep your place.",
            "Smoother loading screens and a fix for a filter carrying over between screens.",
        ],
    },
    Release {
        version: "1.4.0",
        date: "Jul 8, 2026",
        entries: &[
            "The commander picker now leads with the community's most-built commanders, in a fresh order each day.",
            "Partners that name each other now pair automatically.",
            "Star up to three MVP cards per deck, shown on the deck list and while you build.",
            "Share a deck with a public link from its More menu.",
        ],
    },
    Release {
        version: "1.3.1",
        date: "Jul 3, 2026",
        entries: &[
            "Anonymous, PII-free health signals to help us find and fix issues. No account or gameplay data involved.",
            "Internal cleanups for faster, more stable swiping.",
        ],
    },
    Release {
        version: "1.3.0",
        date: "Jul 2, 2026",
        entries: &[
            "Zwipe now remembers your skips: cards you swipe left, or remove from a deck, stay out of that deck's suggestions, saved instantly.",
            "Clear a deck's skips anytime from its More menu.",
            "Every deck remembers your place on the add screen, undo history included.",
            "Swipe down on the empty stack to step back through your last swipes.",
            "Cards sent to the maybeboard now appear there immediately.",
            "Deck card lists now sort alphabetically by default.",
            "Your app version now shows at the bottom of Profile.",
            "Privacy policy updated to cover how your activity personalizes suggestions.",
            "Polish: card images ease in as they load, with consistent spacing on every screen size.",
        ],
    },
    Release {
        version: "1.2.1",
        date: "Jul 1, 2026",
        entries: &[
            "Tap the eye button while swiping to read a card's full rules and stats, handy for alternate-art or textless printings.",
            "The app now opens straight into its themed layout, with no flash of unstyled content.",
        ],
    },
    Release {
        version: "1.2.0",
        date: "Jun 30, 2026",
        entries: &[
            "New draw-odds view: your chance of drawing a land, ramp, removal, and more, by any turn.",
            "Set a power level and add tags like Budget, Jank, Meme, or Precon.",
            "Dozens more strategy tags, each with a plain-language definition.",
            "Toggle synergy suggestions on or off on the add-cards screen.",
            "Contradictory include/exclude filters are now caught, and lands drop out of the add screen once you hit your land target.",
            "Create and edit deck screens now open at the top, plus leader-eligibility fixes.",
        ],
    },
    Release {
        version: "1.1.4",
        date: "Jun 29, 2026",
        entries: &[
            "Set a land target, your own or the format default, with a heads-up if you drop below it.",
            "Filter cards by price (USD, EUR, or Tix).",
            "Set a deck price target with alerts as your total approaches and passes it.",
            "Deck Stats, Distributions, Mana, and Warnings are now tap-to-expand sections.",
            "More consistent filter controls, plus a fix for a flicker on startup.",
        ],
    },
    Release {
        version: "1.1.3",
        date: "Jun 28, 2026",
        entries: &[
            "Card names now show while you swipe, including alternate art and non-English printings.",
            "Tap a field to set your format, commander, or tags, with your deck name checked as you type.",
            "More strategy tags with definitions, plus format and power-level pickers.",
            "Read the Privacy Policy in-app, from Profile.",
            "Sign-in, sign-up, and profile forms now show errors under each field.",
        ],
    },
    Release {
        version: "1.1.2",
        date: "Jun 27, 2026",
        entries: &["Consistency fix so filter controls match across the card-swipe screens."],
    },
    Release {
        version: "1.1.1",
        date: "Jun 26, 2026",
        entries: &[
            "Every screen now has a Help button to report a problem or open our Discord.",
            "New quick tips on the Import and Export screens.",
            "Bug fixes and reliability improvements.",
        ],
    },
    Release {
        version: "1.1.0",
        date: "Jun 24, 2026",
        entries: &[
            "Add up to 5 strategy tags (Aggro, Tokens, Reanimator, and more), shown on the deck list and deck page.",
            "A new \"Zwipe\" option lets you swipe through legal commanders, partners, backgrounds, and signature spells instead of searching.",
            "Tap any deck card to expand it, now with mana cost, type, rules text, and power/toughness or loyalty.",
            "Tap a keyword for a plain-language reminder, and a Keywords button lists every keyword on the current card.",
            "Polish: drop shadows on mana symbols, roomier pips, smoother card details, and scrollable dialogs.",
        ],
    },
    Release {
        version: "1.0.10",
        date: "Jun 23, 2026",
        entries: &[
            "Commander search now shows a \"Searching…\" hint while results load.",
            "Buy links now show an arrow (↗) to signal they open an external store.",
            "A clearer \"update required\" screen.",
            "Small under-the-hood polish.",
        ],
    },
    Release {
        version: "1.0.9",
        date: "Jun 15, 2026",
        entries: &[
            "A brand-new app icon.",
            "Deck list makeover: color-coded detail tags, with the card count turning yellow when a deck is the wrong size for its format.",
            "Oathbreaker, Brawl, Historic Brawl, and Gladiator now use their correct legal deck sizes.",
            "Reorganized Profile: separate Account and Preferences cards, edited in a slide-up sheet, with account deletion under a More menu.",
            "Deck view panels and charts now carry their titles inside each card.",
            "The home-screen quote no longer flickers as you navigate, and refreshes over time.",
            "Polish: \"To mainboard\" button labels, wrapping for long deck names, solid loading placeholders, and warmer Gruvbox text.",
        ],
    },
    Release {
        version: "1.0.7",
        date: "Jun 12, 2026",
        entries: &[
            "A fresh look: a subtle background grid and layered panels across every screen.",
            "Rebuilt theme picker with color swatches, dark mode on top, and colorblind-friendly themes grouped together.",
            "Preferences now opens as a sheet with live preview: Save to keep, back out to revert.",
            "Filters and deck card lists now show colored mana pips instead of letters.",
            "Tap a card's name on the home screen to open its full art.",
            "Cleaner sheets and dialogs with consistent headings and dividers.",
            "Small polish to inputs, chips, spacing, and headers across the app.",
        ],
    },
    Release {
        version: "1.0.6",
        date: "Jun 11, 2026",
        entries: &[
            "Card suggestions now order the add stack by fit with your deck's leader, automatically.",
            "The add-cards screen fills instantly with leader-matched suggestions, no search needed to start.",
            "Cards already in your deck (main, maybeboard, or sideboard) are hidden from suggestions.",
            "Pick any sort (price, mana value, name, and more) to override the smart default.",
            "Import a full deck from a deck-list URL onto your existing deck.",
            "One-time hints on key screens, with a ? button to bring any tip back.",
            "Swipe tips are color-coded by direction for add, skip, maybeboard, and undo.",
            "Empty decks now show one-tap Add cards and Import buttons.",
            "Resend verification now shows a cooldown, plus a Check again button.",
            "Changing your email, username, or password now sends a notification email.",
        ],
    },
    Release {
        version: "1.0.2",
        date: "Jun 8, 2026",
        entries: &[
            "Faster catalog searches, especially on cellular.",
            "In-deck filters now narrow correctly: type, set, format legality, leader eligibility, and rarity.",
            "Card images now have rounded corners everywhere.",
            "Login, sign-up, and edit screens now show loading states.",
            "Show/hide button on every password field.",
            "Loading skeletons replace blank flashes on deck lists, profiles, stats, and home.",
            "Confirmation dialogs darken the background again.",
        ],
    },
    Release {
        version: "1.0.1",
        date: "Jun 7, 2026",
        // Placeholder until the original release notes turn up.
        entries: &["Various performance improvements."],
    },
    Release {
        version: "1.0.0",
        date: "Jun 6, 2026",
        entries: &["Initial release."],
    },
];
