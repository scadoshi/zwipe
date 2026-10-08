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

/// One shipped version and its notes. Dates from the App Store Connect
/// submission history (public "Ready for Distribution" date); newest first.
/// Entries follow `context/development/changelog_guidelines.md`: a phrase
/// saying what changed, no period, no examples. The tests below hold the
/// parts of that a machine can check.
pub struct Release {
    /// Semantic version string, e.g. `"1.6.0"`.
    pub version: &'static str,
    /// Human-readable release date, e.g. `"Jul 12, 2026"`.
    pub date: &'static str,
    /// User-facing release notes, one phrase per entry.
    pub entries: &'static [&'static str],
}

/// Versions in progress for the next release. Rendered at the top of the
/// changelog with an "Upcoming" badge instead of "Latest".
pub const UPCOMING: &[Release] = &[Release {
    version: "1.11.0",
    date: "",
    entries: &[
        "A ? beside the new password field lists the password rules",
        "Privacy policy says where data is stored and how to request a copy",
        "Price and power chips use the theme's palette colors",
        "The chip under the home counters says all users",
        "Screen titles use one capitalization style",
    ],
}];

/// Shipped releases, newest first.
pub const RELEASES: &[Release] = &[
    Release {
        version: "1.10.7",
        date: "Oct 6, 2026",
        entries: &[
            "iOS: buttons respond while a list is still scrolling",
            "iOS: swiping back from the screen edge works while a list is still scrolling",
        ],
    },
    Release {
        version: "1.10.6",
        date: "Oct 5, 2026",
        entries: &["Changing the theme wipes the new one across the screen"],
    },
    Release {
        version: "1.10.5",
        date: "Oct 3, 2026",
        entries: &[
            "The logo resolves in place, without the letters drawing together first",
            "Home shows the mark in a card with the cards swiped, searches run and decks created across everyone",
            "Home says where those counts come from, and says so if they cannot be fetched",
            "Those counts are fetched once a minute at most, however many screens show them",
            "Counts past ten thousand read as 28.0k or 1.2m, so they fit their boxes",
            "Sign in, register and forgot password open on the same card",
            "The Universes Beyond exceptions on Profile ease in one at a time",
        ],
    },
    Release {
        version: "1.10.4",
        date: "Oct 2, 2026",
        entries: &[
            "Deck cards: the commander and MVP row waits for every card image, then opens and deals the cards in",
            "Light themes draw the logo in the theme's primary accent",
            "The logo on the home and sign-in screens resolves from static as the screen opens",
        ],
    },
    Release {
        version: "1.10.3",
        date: "Sep 30, 2026",
        entries: &[
            "Messages appear at the top left, over the title bar, clear of the buttons",
            "Several messages at once stack; tap to open, tap again to close",
            "Messages stay up longer",
            "Save on the Themes, Exceptions, Deck tags, Oracle tags, Format, Filter and Printing sheets is greyed out until something changes; Back puts things back",
            "Swiping back out of the Oracle tags picker drops your edits, the same as Back",
            "Dark mode and Universes Beyond say what they switched to",
            "Every deck form field and most Profile rows have their own ?",
            "Shorter hints across the app",
            "Commander select no longer offers cards that are only a legendary creature on their back face, or melded cards; a deck that already has one warns",
            "Oathbreaker signature spell select no longer offers Adventure creatures; split cards still qualify",
        ],
    },
    Release {
        version: "1.10.2",
        date: "Sep 22, 2026",
        entries: &[
            "Clearing a filter on the add screen works; Apply used to refuse an empty filter",
            "Applying the filter unchanged leaves the card stack where it was",
            "Shaking the phone no longer opens the system Undo prompt",
            "When the filter lists can't load, the app says so and tries again",
            "The command zone strip no longer flashes placeholders or jumps as images load",
            "The welcome message greets you once per launch, not every return to Home",
        ],
    },
    Release {
        version: "1.10.1",
        date: "Sep 6, 2026",
        entries: &[
            "Every failure while talking to the server says so, with the same brief message on every screen",
            "A failed card batch while swiping says so instead of the pile running dry",
            "The Decks screen and the commander maybeboard show a note chip when their lists can't load",
            "Links on zwipe.net open in the same tab; Ctrl or middle click still opens a new one",
        ],
    },
    Release {
        version: "1.10.0",
        date: "Sep 1, 2026",
        entries: &[
            "A Universes Beyond setting in Profile hides crossover cards from searches and commander picks, with per-franchise exceptions",
            "Set filters check every printing of a card",
            "Searches show a card's in-universe printing when it has one",
            "An empty search says why",
        ],
    },
    Release {
        version: "1.9.3",
        date: "Aug 20, 2026",
        entries: &[
            "Cards sharing a role with your deck's MVPs come up sooner while you swipe",
            "Sign-in no longer rejects a password for failing the current password rules",
            "Changing your email prompts you to verify the new address",
            "Importing a starred card onto another board no longer keeps its MVP star",
            "Choosing a printing while swiping no longer reports it as saved",
            "Corrected in-app help and messages across the deck, card, filter and import screens",
            "Corrected the guides on zwipe.net; every oracle tag has a Zwipe-written description",
        ],
    },
    Release {
        version: "1.9.2",
        date: "Aug 18, 2026",
        entries: &[
            "Decks show their command zone's art on the Decks screen",
            "The Decks screen is one connected list; Group by splits it into foldable sections",
            "Grouping a deck's cards by color gives each color combination its own group",
            "Color group headers show mana pips instead of color names",
            "Back closes what's open instead of leaving the screen",
            "Android: fixed a crash when opening Zwipe from a notification, another app or the Play Store while it was already running",
            "Android: switching between light and dark mode no longer closes the app",
            "The guides page on zwipe.net has a search bar",
            "The commander and MVP cards on a shared deck page deal in from above",
            "Deck list color group headers show the same size pips as the decks under them",
        ],
    },
    Release {
        version: "1.9.1",
        date: "Aug 14, 2026",
        entries: &[
            "Keyword reminders for the newest sets explain the mechanic instead of pointing at the card",
            "Keyword definitions update on their own, no app update needed",
            "Tapping a card role tag on a commander maybeboard entry shows its description and Examples",
            "Evened out the spacing on the deck list console and the deck profile",
        ],
    },
    Release {
        version: "1.9.0",
        date: "Aug 13, 2026",
        entries: &[
            "Swipe up while picking a commander to save it to your commander maybeboard",
            "The commander maybeboard opens from More on the Decks screen, with swipe and quick add",
            "Maybeboard entries expand for details, start a deck, or remove",
            "Expanded card rows inside grouped lists no longer stretch past the screen",
            "Shared deck pages carry the app's deck sections, collapsed until wanted",
            "Card groups on shared deck pages collapse from their headers",
        ],
    },
    Release {
        version: "1.8.1",
        date: "Aug 12, 2026",
        entries: &[
            "Tap a group header on your deck's card list to collapse it",
            "Search finds cards whose newer foreign printing was hiding the English one",
            "Loading screens for the deck list and deck cards match the real layouts",
            "The commander and MVP cards at the top of the deck deal in from above",
            "Deck list group headers drop their underline",
        ],
    },
    Release {
        version: "1.8.0",
        date: "Aug 12, 2026",
        entries: &[
            "The deck list has Group by and Show rows",
            "A one-time tip on the deck list covers the new rows; ? brings it back",
            "The import and export screens keep their controls in place while results scroll underneath",
            "Bolder chip row labels",
        ],
    },
    Release {
        version: "1.7.6",
        date: "Aug 10, 2026",
        entries: &[
            "Featured flavor is the same card for everyone, picked fresh every hour, on the app and zwipe.net",
            "Undo reaches every way you change your deck",
            "Importing a decklist starts the deck's undo history fresh",
            "Every search bar has a clear button",
            "Quick add finds cards you skipped while swiping",
            "Android: reopening the app after the system closed it in the background starts clean instead of crashing",
            "Rate limit messages say how long to wait",
            "Card roles read only real rules text",
            "Tighter zwipe.net home page",
        ],
    },
    Release {
        version: "1.7.5",
        date: "Aug 4, 2026",
        entries: &[
            "Undo on your deck's card list, one change at a time",
            "Quick add by card name at the top of your deck's card list",
            "Your deck's name, format, power level and tags show at the top of the card list",
            "Search suggestions float over the screen instead of pushing it around",
            "The command zone and starred MVPs show as card images across the top of your deck's card list",
            "Every card row shows its artwork; the Art chip turns it off",
            "The Maybe and Side board buttons gray out when those boards are empty",
            "Swiped cards fly out from where you let go",
            "Swipes need a fuller drag or a real flick to commit",
            "A card released before the swipe point settles back with a bounce",
            "On the site, the hover card preview hops to the far side of the screen",
            "Rapid quantity taps batch into one update that is safe to retry",
        ],
    },
    Release {
        version: "1.7.4",
        date: "Jul 30, 2026",
        entries: &[
            "Fixed a crash when saving a card image to your photos from a long press",
            "Anonymous error and crash reports, with no account or personal data attached",
        ],
    },
    Release {
        version: "1.7.3",
        date: "Jul 24, 2026",
        entries: &[
            "Filter edits wait for Apply; Reset, Cancel and tapping outside each say what they did",
            "Average power and toughness in the Distributions section",
            "Oracle tag search puts exact matches first and searches descriptions",
            "Shared deck pages keep card groups in order on phones",
            "Tap an oracle tag in a card's details to read its definition while you swipe",
        ],
    },
    Release {
        version: "1.7.2",
        date: "Jul 20, 2026",
        entries: &[
            "Long oracle and deck tags no longer overlap their labels in the deck's Tags section",
            "The Category grouping is now Card role, matching the rest of the app",
            "The card filter applies to the Maybeboard and Sideboard too",
            "Lands sit in their own section at the bottom of the deck list in every grouping",
            "Shared deck pages can show the tokens your cards make",
            "Keyword reminders for the Avatar bending abilities describe what each does",
            "Tap outside any dialog to dismiss it",
            "Clearer in-app guides with color-coded button names; the oracle-tag example cards have their own guide",
        ],
    },
    Release {
        version: "1.7.1",
        date: "Jul 17, 2026",
        entries: &[
            "Cards without a printed image show as a text card",
            "Example cards for any oracle tag, from the dictionary",
            "Use a tag straight from the dictionary in your deck's strategy or your card filter",
            "Every oracle tag has a plain-language description",
            "Tap an oracle tag on one of your cards to read it there",
            "Card details restyled to match the rest of the app and open scrolled to the top",
            "The back gesture closes an open dictionary, picker or filter one layer at a time",
            "Hidden scrollbars and soft fade edges on lists and dialogs",
        ],
    },
    Release {
        version: "1.7.0",
        date: "Jul 14, 2026",
        entries: &[
            "Oracle-tag dictionary, opened from the oracle-tag picker",
            "Better tags under each card role and over a thousand plain-language descriptions",
            "Filters and pickers open instantly",
            "Flip double-faced cards from the card details",
            "The Export screen shows a loading placeholder instead of a spinner",
            "The changelog updates on its own",
        ],
    },
    Release {
        version: "1.6.0",
        date: "Jul 12, 2026",
        entries: &[
            "In-app changelog, from your Profile",
            "Cleaner deck card screen with squircle mana pips, an inline price tag and power/toughness on each row",
            "Theme and dark-mode controls moved to your Profile",
            "Seventeen new themes, thirty-one in all",
            "An Achromatopsia theme, plus a contrast pass on every theme",
            "Your theme persists across the app and website",
            "Buy a card from the home screen by tapping its price",
            "Fixed importing double-faced cards",
            "Light-mode polish across every theme",
            "Oracle tags: community-maintained tags for what cards do",
            "Card roles on every card, with the oracle tags underneath",
            "Choose oracle tags for your deck's strategy, or pick an archetype to seed them",
            "In-app guides for deck tags, card roles and oracle tags",
            "Deck color identity shows as mana pips next to each deck's name",
            "Every mana pip tints its outline and glyph to its own color in any theme",
            "Deck view reorganized into collapsible Profile, Budget and Tags sections",
            "Deck lands moved to the Mana section, shown against your land target",
            "Shared deck pages show card roles and the full deck price",
            "Centered hybrid mana symbols, clearer section labels and smoother card-detail reveals",
        ],
    },
    Release {
        version: "1.5.0",
        date: "Jul 10, 2026",
        entries: &[
            "Swipe back from the left edge to return to the previous page",
            "Every screen keeps its own filters per deck",
            "An expanded card stays highlighted so you keep your place",
            "Smoother loading screens; a filter no longer carries over between screens",
        ],
    },
    Release {
        version: "1.4.0",
        date: "Jul 8, 2026",
        entries: &[
            "The commander picker leads with the community's most-built commanders, in a fresh order each day",
            "Partners that name each other pair automatically",
            "Star up to three MVP cards per deck",
            "Share a deck with a public link from its More menu",
        ],
    },
    Release {
        version: "1.3.1",
        date: "Jul 3, 2026",
        entries: &[
            "Anonymous diagnostics, with no account or gameplay data involved",
            "Internal cleanups for faster, more stable swiping",
        ],
    },
    Release {
        version: "1.3.0",
        date: "Jul 2, 2026",
        entries: &[
            "Zwipe remembers your skips per deck",
            "Clear a deck's skips from its More menu",
            "Every deck remembers your place on the add screen, undo history included",
            "Swipe down on the empty stack to step back through your last swipes",
            "Cards sent to the maybeboard appear there immediately",
            "Deck card lists sort alphabetically by default",
            "Your app version shows at the bottom of Profile",
            "Privacy policy updated to cover how your activity personalizes suggestions",
            "Card images ease in as they load, with consistent spacing on every screen size",
        ],
    },
    Release {
        version: "1.2.1",
        date: "Jul 1, 2026",
        entries: &[
            "Tap the eye button while swiping to read a card's full rules and stats",
            "The app opens straight into its themed layout",
        ],
    },
    Release {
        version: "1.2.0",
        date: "Jun 30, 2026",
        entries: &[
            "Draw-odds view: your chance of drawing a land, ramp, removal and more, by any turn",
            "Set a power level and add deck tags",
            "Dozens more strategy tags, each with a plain-language definition",
            "Toggle synergy suggestions on the add-cards screen",
            "Contradictory include/exclude filters are caught; lands drop out of the add screen once you hit your land target",
            "Create and edit deck screens open at the top; leader-eligibility fixes",
        ],
    },
    Release {
        version: "1.1.4",
        date: "Jun 29, 2026",
        entries: &[
            "Set a land target, your own or the format default, with a heads-up if you drop below it",
            "Filter cards by price in USD, EUR or Tix",
            "Set a deck price target with alerts as your total approaches and passes it",
            "Deck Stats, Distributions, Mana and Warnings are tap-to-expand sections",
            "More consistent filter controls; fixed a flicker on startup",
        ],
    },
    Release {
        version: "1.1.3",
        date: "Jun 28, 2026",
        entries: &[
            "Card names show while you swipe, including alternate art and non-English printings",
            "Tap a field to set your format, commander or tags, with your deck name checked as you type",
            "More strategy tags with definitions, plus format and power-level pickers",
            "Read the Privacy Policy in-app, from Profile",
            "Sign-in, sign-up and profile forms show errors under each field",
        ],
    },
    Release {
        version: "1.1.2",
        date: "Jun 27, 2026",
        entries: &["Filter controls match across the card-swipe screens"],
    },
    Release {
        version: "1.1.1",
        date: "Jun 26, 2026",
        entries: &[
            "Every screen has a Help button to report a problem or open our Discord",
            "Quick tips on the Import and Export screens",
            "Bug fixes and reliability improvements",
        ],
    },
    Release {
        version: "1.1.0",
        date: "Jun 24, 2026",
        entries: &[
            "Add up to 5 strategy tags, shown on the deck list and deck page",
            "A Zwipe option swipes through legal commanders, partners, backgrounds and signature spells instead of searching",
            "Tap any deck card to expand it, with mana cost, type, rules text and power/toughness or loyalty",
            "Tap a keyword for a plain-language reminder; a Keywords button lists every keyword on the card",
            "Drop shadows on mana symbols, roomier pips, smoother card details and scrollable dialogs",
        ],
    },
    Release {
        version: "1.0.10",
        date: "Jun 23, 2026",
        entries: &[
            "Commander search shows a Searching hint while results load",
            "Buy links show an arrow to signal they open an external store",
            "A clearer update-required screen",
            "Small under-the-hood polish",
        ],
    },
    Release {
        version: "1.0.9",
        date: "Jun 15, 2026",
        entries: &[
            "A new app icon",
            "Deck list redone with color-coded detail tags; the card count turns yellow when a deck is the wrong size for its format",
            "Oathbreaker, Brawl, Historic Brawl and Gladiator use their correct legal deck sizes",
            "Profile reorganized into Account and Preferences cards, edited in a slide-up sheet, with account deletion under More",
            "Deck view panels and charts carry their titles inside each card",
            "The home-screen quote no longer flickers as you navigate, and refreshes over time",
            "To mainboard button labels, wrapping for long deck names, solid loading placeholders and warmer Gruvbox text",
        ],
    },
    Release {
        version: "1.0.7",
        date: "Jun 12, 2026",
        entries: &[
            "A subtle background grid and layered panels across every screen",
            "Rebuilt theme picker with color swatches, dark mode on top and colorblind-friendly themes grouped together",
            "Preferences opens as a sheet with live preview",
            "Filters and deck card lists show colored mana pips instead of letters",
            "Tap a card's name on the home screen to open its full art",
            "Cleaner sheets and dialogs with consistent headings and dividers",
            "Small polish to inputs, chips, spacing and headers across the app",
        ],
    },
    Release {
        version: "1.0.6",
        date: "Jun 11, 2026",
        entries: &[
            "Card suggestions order the add stack by fit with your deck's leader",
            "The add-cards screen fills instantly with leader-matched suggestions",
            "Cards already in your deck are hidden from suggestions",
            "Pick any sort to override the smart default",
            "Import a full deck from a deck-list URL onto your existing deck",
            "One-time hints on key screens, with a ? button to bring any tip back",
            "Swipe tips are color-coded by direction",
            "Empty decks show one-tap Add cards and Import buttons",
            "Resend verification shows a cooldown, plus a Check again button",
            "Changing your email, username or password sends a notification email",
        ],
    },
    Release {
        version: "1.0.2",
        date: "Jun 8, 2026",
        entries: &[
            "Faster catalog searches, especially on cellular",
            "In-deck filters narrow correctly by type, set, format legality, leader eligibility and rarity",
            "Card images have rounded corners everywhere",
            "Login, sign-up and edit screens show loading states",
            "Show/hide button on every password field",
            "Loading skeletons replace blank flashes on deck lists, profiles, stats and home",
            "Confirmation dialogs darken the background again",
        ],
    },
    Release {
        version: "1.0.1",
        date: "Jun 7, 2026",
        // Placeholder until the original release notes turn up.
        entries: &["Various performance improvements"],
    },
    Release {
        version: "1.0.0",
        date: "Jun 6, 2026",
        entries: &["Initial release"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn every_entry() -> impl Iterator<Item = (&'static str, &'static str)> {
        UPCOMING.iter().chain(RELEASES).flat_map(|release| {
            release
                .entries
                .iter()
                .map(move |entry| (release.version, *entry))
        })
    }

    /// An entry is a phrase, not a sentence: no period, no exclamation, no
    /// trailing ellipsis.
    #[test]
    fn no_entry_ends_with_a_period() {
        let offenders: Vec<_> = every_entry()
            .filter(|(_, entry)| entry.ends_with(['.', '!', '\u{2026}']))
            .collect();
        assert!(offenders.is_empty(), "{offenders:#?}");
    }

    /// Examples belong in the guides. An entry says what changed and stops.
    #[test]
    fn no_entry_gives_an_example() {
        let offenders: Vec<_> = every_entry()
            .filter(|(_, entry)| {
                entry.contains(" like ")
                    || entry.contains("e.g.")
                    || entry.contains("for example")
                    || entry.contains("such as")
            })
            .collect();
        assert!(offenders.is_empty(), "{offenders:#?}");
    }

    #[test]
    fn no_entry_is_empty_or_uses_an_em_dash() {
        let offenders: Vec<_> = every_entry()
            .filter(|(_, entry)| entry.trim().is_empty() || entry.contains('\u{2014}'))
            .collect();
        assert!(offenders.is_empty(), "{offenders:#?}");
    }
}
