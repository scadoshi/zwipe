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
    version: "1.10.0",
    date: "",
    entries: &[
        "A Universes Beyond setting in Profile hides crossover cards from searches and commander picks. Checking a franchise there keeps it.",
        "Set filters check every printing of a card. Including a set matches everything printed there; excluding one no longer hides cards also printed elsewhere.",
        "Searches show a card's in-universe printing when it has one, instead of its newest crossover printing.",
        "An empty search says why: the filter matched nothing, or Synergy is on and limiting results.",
    ],
}];

/// Shipped releases, newest first.
pub const RELEASES: &[Release] = &[
    Release {
        version: "1.9.3",
        date: "Aug 20, 2026",
        entries: &[
            "Cards sharing a role with your deck's MVPs now come up sooner while you swipe.",
            "Sign-in no longer rejects a password for failing the current password rules.",
            "Changing your email now prompts you to verify the new address.",
            "Importing a starred card onto another board no longer keeps its MVP star.",
            "Choosing a printing while swiping no longer reports it as saved.",
            "Corrected in-app help and messages across the deck, card, filter, and import screens.",
            "Corrected the guides on zwipe.net, and every oracle tag now has a Zwipe-written description.",
        ],
    },
    Release {
        version: "1.9.2",
        date: "Aug 18, 2026",
        entries: &[
            "Your decks now show their command zone's art on the Decks screen: the commander, both partners, or the oathbreaker and its signature spell.",
            "The Decks screen is one connected list. Turning on Group by splits it into sections you can fold from their headers.",
            "Grouping a deck's cards by color now gives each color combination its own group, instead of one Multicolor pile.",
            "Color group headers show mana pips instead of color names, on your deck's cards, the deck list, and shared deck pages.",
            "Back now closes what's open instead of leaving the screen. This covers the format and tag pickers, the printing sheet, and the More sheets.",
            "Android: fixed a crash when opening Zwipe from a notification, another app, or the Play Store while it was already running.",
            "Android: switching your phone between light and dark mode no longer closes the app.",
            "The guides page on zwipe.net has a search bar. It matches a guide's title, summary, tags, and text.",
            "The commander and MVP cards on a shared deck page deal in from above as they load.",
            "Deck list color group headers show the same size pips as the decks under them.",
        ],
    },
    Release {
        version: "1.9.1",
        date: "Aug 14, 2026",
        entries: &[
            "Keyword reminders for the newest sets: Blight, Prepared, Vivid, Start your engines!, and two dozen more now explain the actual mechanic instead of pointing you at the card.",
            "Keyword definitions now update on their own, no app update needed.",
            "Tapping a card role tag on a commander maybeboard entry now shows its description, with the same Examples button as the deck card list.",
            "The deck list console sits at the same distance from the header as other screens, and the deck profile's section spacing evens out.",
        ],
    },
    Release {
        version: "1.9.0",
        date: "Aug 13, 2026",
        entries: &[
            "Swipe up while picking a commander to save it to your commander maybeboard.",
            "Open the maybeboard from More on the Decks screen. Swipe deals commanders right there: right starts a deck with the pick, up saves it for later. Quick add saves commanders by name.",
            "Maybeboard entries expand for details; Create deck starts a new deck with that commander and Remove drops it.",
            "Expanded card rows inside grouped lists no longer stretch past the screen.",
            "Shared deck pages on zwipe.net now carry the app's deck sections: Budget, Tags, Distributions, Mana, and Draw odds sit between the featured cards and the list, collapsed until you want them.",
            "Card groups on shared deck pages collapse from their headers too, with the same arrow and ease as the app.",
        ],
    },
    Release {
        version: "1.8.1",
        date: "Aug 12, 2026",
        entries: &[
            "Tap a group header on your deck's card list to collapse it, with the same arrow card rows use. Tokens, Lands, Maybeboard, and Sideboard collapse too.",
            "Search finds cards like Arcane Signet again: a newer foreign-language printing could hide a card's English version from name search. English printings win now.",
            "The loading screens for the deck list and deck cards match the real layouts, card images and chip rows included, and the controls that never load (Group by, Show, the color pips) render right away instead of ghosting.",
            "The commander and MVP cards at the top of the deck deal in from above, one after another.",
            "Deck list group headers drop their underline.",
        ],
    },
    Release {
        version: "1.8.0",
        date: "Aug 12, 2026",
        entries: &[
            "The deck list now has Group by and Show rows: group your decks into sections by format, color, or tag, and filter them with color and tag chips.",
            "A one-time tip on the deck list covers the new rows. The ? button brings it back whenever you want.",
            "The import and export screens keep their controls in place while results scroll underneath, fading out at the edges like the dictionary list.",
            "The small chip row labels (From:, Board:, Group by:, and the rest) are bolder, so they hold up against the background grid.",
        ],
    },
    Release {
        version: "1.7.6",
        date: "Aug 10, 2026",
        entries: &[
            "Featured flavor: the home screen quote is now the same card for everyone, picked fresh every hour from the past year of sets. The same card shows on zwipe.net, where tapping its name reveals the image.",
            "Undo now reaches every way you change your deck: swipes on the add and remove screens and command zone printing swaps all step back from the Undo button on your deck's card list.",
            "Importing a decklist starts the deck's undo history fresh.",
            "Every search bar now has a clear button: one tap wipes what you typed and closes the results.",
            "Quick add now finds cards you skipped while swiping, so passing on a card no longer hides it from a name search. The swipe pile still honors your skips, and Clear skips still brings them back there.",
            "Android: reopening the app after the system quietly closed it in the background now starts clean instead of crashing.",
            "Rate limit messages now say how long to wait in plain terms, like \"try again in a minute\".",
            "Card roles now read only real rules text: split second spells are no longer labeled stax, and cards that exile from your own graveyard as a cost are no longer labeled graveyard hate.",
            "The zwipe.net home page tightened up: the demo, the three core features, and the hour's featured flavor now share the screen, and reviews live in one panel.",
        ],
    },
    Release {
        version: "1.7.5",
        date: "Aug 4, 2026",
        entries: &[
            "Undo on your deck's card list: steps back adds, removals, quantity changes, board moves, and printing swaps one at a time, naming the card each time.",
            "Quick add: type a card name at the top of your deck's card list and tap a result to add it. Cards already in your deck stay out of the results.",
            "Your deck's name, format, power level, and tags now show at the top of the card list, matching the shared deck page.",
            "Search suggestions in the filters and pickers now float over the screen instead of pushing it around while you type.",
            "The command zone and your starred MVPs now show as card images across the top of your deck's card list, like the shared deck page. Tap one to see it full size.",
            "Every card row now shows its artwork. The Art chip in the Show row turns it off, on the shared deck page too.",
            "The Maybe and Side board buttons gray out when those boards are empty.",
            "Swiped cards now fly out from right where you let go, instead of snapping back to center first.",
            "Swipes are harder to trigger by accident: a card needs a fuller drag or a real flick to commit.",
            "A card released before the swipe point now settles back to center with a little bounce.",
            "On the site, the hover card preview now hops to the far side of the screen instead of sitting under your cursor.",
            "Under the hood: rapid quantity taps batch into a single update, sent in a format that is safe to retry.",
        ],
    },
    Release {
        version: "1.7.4",
        date: "Jul 30, 2026",
        entries: &[
            "Fixed a crash when saving a card image to your photos from a long press. Saving now asks for permission and works.",
            "The app now sends anonymous error and crash reports, so problems get found and fixed faster. No account or personal data rides along.",
        ],
    },
    Release {
        version: "1.7.3",
        date: "Jul 24, 2026",
        entries: &[
            "Filter edits now wait for Apply: Reset stages the default, Cancel or tapping outside restores your filter, and a toast confirms what happened either way.",
            "See your deck's average power and toughness in the Distributions section.",
            "Oracle tag search now puts exact matches first and searches descriptions too, in the deck picker, the card filter, and the dictionary.",
            "Shared deck pages keep card groups in order on phones.",
            "Tap an oracle tag in a card's details to read its definition while you swipe.",
        ],
    },
    Release {
        version: "1.7.2",
        date: "Jul 20, 2026",
        entries: &[
            "Fixed the deck's Tags section so long oracle and deck tags no longer overlap their labels.",
            "The deck cards grouping once labeled Category now reads Card role, matching the filter and the rest of the app.",
            "Your card filter now applies to the Maybeboard and Sideboard, not just the main deck.",
            "Lands now sit in their own section at the bottom of the deck list, in every grouping. Hide them with the card filter.",
            "Shared deck pages can now show the tokens your cards make.",
            "The keyword reminders for the Avatar bending abilities now describe what each actually does.",
            "Tap outside any dialog to dismiss it.",
            "The in-app guides are clearer and color-code the buttons they mention, and the oracle-tag example cards now have their own guide.",
        ],
    },
    Release {
        version: "1.7.1",
        date: "Jul 17, 2026",
        entries: &[
            "Cards without a printed image now show as a text card with type, mana cost, rules, and power or toughness.",
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
            "Better tags under each card role, closer archetype matches, and over a thousand plain-language descriptions.",
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
            "Every screen now keeps its own filters per deck.",
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
            "Anonymous diagnostics to help find and fix issues. No account or gameplay data involved.",
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
            "A new app icon.",
            "Deck list redone: color-coded detail tags, with the card count turning yellow when a deck is the wrong size for its format.",
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
            "A new look: a subtle background grid and layered panels across every screen.",
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
