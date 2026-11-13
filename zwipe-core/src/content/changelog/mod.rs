//! Shared changelog data.
//!
//! The release history, compiled into every surface: the website (`zite`) and
//! the app (`zwiper`) render it, and the server (`zerver`) serves it at
//! `/api/changelog` so new clients can fetch fresh entries without an app
//! resubmit. This is the single source of truth; the wire types in
//! [`crate::http::contracts::changelog`] project it into an owned, serializable
//! response.
//!
//! Data lives here (pure, no UI deps) rather than in `components` so the
//! server can serve it without depending on a UI crate.

use chrono::NaiveDate;

use crate::version::Version;

/// How a release date reads on every surface, e.g. `"Oct 9, 2026"`. Also the
/// format of the wire's `date` string, which apps up to 1.13.0 show as is.
pub const DISPLAY_DATE_FORMAT: &str = "%b %-d, %Y";

/// One version and its notes, dated the day its build was cut and submitted
/// to the stores; newest first. Entries
/// follow `context/development/changelog_guidelines.md`: a phrase saying what
/// changed, no period, no examples. The tests below hold the parts of that a
/// machine can check.
#[derive(Debug)]
pub struct Release {
    /// The release number.
    pub version: Version,
    /// The day its build was cut and submitted, not the day it went live.
    /// `None` only for [`UPCOMING`], which has not been cut.
    /// `NaiveDate::from_ymd_opt` keeps this const; a test checks every
    /// shipped release got a valid date.
    pub date: Option<NaiveDate>,
    /// User-facing release notes, one phrase per entry.
    pub entries: &'static [&'static str],
}

/// The version last submitted to the App Store and Play: the changelog
/// marks it Mobile, and a newer shipped release Web, since the web deploys
/// ahead of the stores. Bump it when the mobile build is submitted, on
/// submission day, not when review clears; until Apple and Google release
/// it, a phone may still offer the version before. Compiled in, not
/// served: each client reads its own copy.
pub const MOBILE_STORE_VERSION: Version = Version::new(1, 18, 15);

/// Versions in progress for the next release. Rendered at the top of the
/// changelog with an "Upcoming" badge instead of "Latest".
pub const UPCOMING: &[Release] = &[
];

/// Shipped releases, newest first.
pub const RELEASES: &[Release] = &[
    Release {
        version: Version::new(1, 20, 5),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web and zwipe.net, t opens Themes on every screen, where the arrow keys and h, j, k and l move through the themes and Enter picks one",
            "On the web and zwipe.net, gg and Shift+G glide to the top or bottom, and j and k scroll a short step where the screen doesn't use them, at any window width",
            "Inside a sheet, gg and Shift+G go to its first or last item",
            "Shift+H opens the keyboard shortcuts on every screen, in two columns on a wide window",
            "On the web, Esc closes only the sheet on top",
            "Cloning a deck on the web selects the copy's name, so typing replaces it",
        ],
    },
    Release {
        version: Version::new(1, 20, 4),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "The web app loads faster the first time after an update, its download a third smaller, and later visits reuse what the browser already has",
            "A shared deck's link preview credits the art with the artist tag Printings shows",
            "On the web, Enter runs an open sheet's main action, or closes the sheet when nothing changed",
            "On the web, a dialog opens with the cursor in its first field or on its main button, and hands focus back when it closes",
            "Printings names the card at its top, and on a phone shows each printing's set, number and place as well as its artist, on the web and in the app",
            "A filter section opens with its choices appearing whole and in place, so a quick tap lands where you aimed, on the web and in the app",
            "On the web, a page left open says when a new version is out, with a Reload button",
        ],
    },
    Release {
        version: Version::new(1, 20, 3),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "Quick add offers the most played cards first among names that match as closely",
            "On the Remove screen, undoing a move to the maybeboard puts the card's MVP star back",
            "Undoing a move to the maybeboard or sideboard puts the card's MVP star back",
            "Undo on a deck's cards says what it undid only once the change is saved, on the web and in the app",
            "On the web, the Decks list opens and closes its Tags without the decks jittering",
            "On the web's deck page, holding a key to run down the cards swaps the details drawer at once instead of fading each card",
        ],
    },
    Release {
        version: Version::new(1, 20, 2),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On a phone, a deck's Profile and Cards come and go as screens do, each entering fresh with nothing showing beneath, on the web and in the app",
            "On phone web, a deck's Cards opened from a link show Cards in the header",
            "On a phone, a card's details drawer on a deck's Cards dims the page and closes on a tap outside, its x, or a drag to the right, on the web and in the app",
            "On a phone, a card's details drawer scrolls its own content with a vertical swipe, and an opened card role or tag scrolls into view",
            "A card's details drawer on a deck's Cards opens on the left when a right one would cover the card, and on a phone drags toward its side to close",
            "On a phone's web app, the featured cards are the app's: big, scrolling sideways, and nothing under them moves when one joins or leaves",
            "A List group's price and cart sit at the end of its title's line on a phone, on the web and in the app",
        ],
    },
    Release {
        version: Version::new(1, 20, 1),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web's deck page, the arrow keys or h, j, k and l move between cards and Enter opens one, in List and Stacks",
            "On the web's deck page, the arrow keys and h, j, k and l move between cards at any window width, a tablet's keyboard included",
            "On the web's deck page, the card keys act on the card the details drawer shows, wherever it opened from",
            "On the web, Esc closes Quick add's suggestions and leaves the box, so the card keys work again",
            "On the web, Back closes the open sheets one at a time before it leaves the page, on a phone too",
            "On the web, Caps Lock no longer changes what a letter shortcut does",
            "On the web, an Oracle tag's Examples keep Next, Back one and the details eye on one bar",
            "On the web's deck page, a featured card's details leave the page undimmed in List as in Stacks",
            "On the web, the Commander maybeboard, Activity and Playstyle take the arrow keys or h, j, k and l to move between cards, with the card's image following, and Enter opens one",
        ],
    },
    Release {
        version: Version::new(1, 20, 0),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "A deck opens on its Profile with Cards in the dock, which opens the cards over it and closes with Back, for your decks and shared ones, on the web and in the app",
            "A deck's Cards slide in over its Profile and slide back out, and an edit made there shows on the Profile right away, on the web and in the app",
            "A shared deck on a phone opens on the same Profile as your own decks, and every shared deck shows its warnings and offers Export",
            "On a phone, a deck's Cards open and close at once over the grid tiles, on the web and in the app",
            "On a phone, the featured cards deal in each time you open a deck's Cards, without moving anything under them",
            "On phone web, scrolling a deck's Cards lets them fill the space the header leaves",
            "On a phone's web app, the menu opens over a deck's Cards",
            "On a phone's web app, the featured cards keep a gap above Quick add",
        ],
    },
    Release {
        version: Version::new(1, 19, 12),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "A featured card's details no longer lift the same card in Stacks",
            "On the web, the signed-out nav offers Log in and Create account, with Forgot password on the Log in screen",
            "On the web at phone width, Printings takes the arrow keys and Enter as soon as it opens",
            "On the web at phone width, the filter's title bar pulls it down to close, as Printings' does",
            "On the web, Change username, Change email and Change password put the cursor in their first field as they open",
            "Screen readers announce an open sheet as a dialog named by its title",
            "In Stacks, a tapped card eases open and the revealed card drifts in softly, on the web and in the app",
        ],
    },
    Release {
        version: Version::new(1, 19, 11),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On a phone, a deck's featured card opens its details on the first tap, and the strip no longer cuts off a card's shadow",
            "A warning's Fix to on a deck's profile can be undone from its cards, on the web and in the app",
            "Picking a printing keeps the card's details open on that card",
            "Moving a card to the maybeboard or sideboard says how many copies moved",
            "Quick add lands a card's cheapest regular printing instead of its newest",
            "Quick add says when the card you search for is already in the deck, and lists the closest names first",
            "The featured cards and the cards under them ease into place when a star adds or takes one",
        ],
    },
    Release {
        version: Version::new(1, 19, 10),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "A deck's featured cards open their details and take the card keys as the same cards in the list do, on the web and in the app",
            "A new username can't be one of Zwipe's own names or one that differs from another account's only in case",
            "The sheet you opened last sits on top of the others, and on a wide window Themes and the filter leave the page undimmed",
            "zwipe.net answers a page that isn't there with Page not found and a not-found status, so search engines leave it out",
            "On zwipe.net, Esc closes the featured card's art as a click does",
            "On a phone or a narrow window, the web's deck cards get the View chip and Stacks as the app has them, and each List group shows its price and cart",
            "Undo on a deck's cards puts back a star or an unstar, on the web and in the app",
        ],
    },
    Release {
        version: Version::new(1, 19, 9),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "Printings loads as one fan of card ghosts, with ghosts of its chips, on the web and in the app",
            "Printings runs its cards out to the sheet's edges with no fade, its arrows over the cards, on the web and in the app",
            "On the web's deck page, a featured card lifts and sways gently under the mouse",
            "zwipe.net's changelog moves to About, linked beside Privacy policy and in the footer instead of the top nav",
            "A shared deck's link opens on app.zwipe.net, signed in or not, with its preview in chats as before",
            "In Stacks, the card you reveal lifts and sways gently instead of taking an outline, on the web and in the app",
            "In a deck's Stacks, an Oracle tag in the open card's drawer offers its Examples, as in the List",
        ],
    },
    Release {
        version: Version::new(1, 19, 8),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "In the app, each card group in a deck's List has its price and a cart that buys its cards, as its stack does",
            "On the web's deck page, each column of Stacks runs on its own, every stack right under the one above it",
            "zwipe.net's shared deck is the web app's deck page, read only",
            "The preview cards for zwipe.net and app.zwipe.net links no longer underline Magic: The Gathering",
            "On a phone, zwipe.net's shared deck page shows its profile or its cards, under a Profile and Cards switch",
            "A card's Image, Flip, Meld and Star buttons and the Printings Select are icons, the star filling in gold when the card is starred",
            "A Stacks group header wraps a long name instead of cutting it short, with the cart right after the price, on the web and in the app",
            "On zwipe.net's shared deck, the owner's name is a chip beside the deck's name",
            "A starred card's mark and its MVP chips share the star's gold",
            "In light themes, a chip that can't be used right now fades as it does in dark themes",
        ],
    },
    Release {
        version: Version::new(1, 19, 7),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web, cards, decks and guides ease into place when a fold, a Show or Sort chip, a filter, a row's action or Undo moves them, instead of jumping",
            "The nav bar, a field's label and the Draw odds bars no longer jump when they change",
            "On the web, Home pairs Decks and Colors under the hero, with Featured flavor on its own row",
            "On a phone, a deck's Edit, Clone and Share sit in the dock beside Cards",
            "zwipe.net's videos and guide screenshots hold their place as a pulsing placeholder while they load, then fade in",
            "On the web, Home's Colors panel runs as tall as Decks beside it",
            "zwipe.net draws its logos in the same font as the app, so they look the same",
        ],
    },
    Release {
        version: Version::new(1, 19, 6),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web, the Commander maybeboard takes p, i, v and d on a card, as Playstyle and Activity do",
            "The Commander maybeboard sorts by Newest saved, Name, Mana value or Color, and remembers the order",
            "The web's Home and its other few-panel pages sit in the same 1400px column as zwipe.net",
            "On the web, Import runs the full width of its page, as Export does",
            "On the web, Home's tagline and the Privacy policy's text run the full width of their panels",
            "Back closes an open card image before leaving the screen",
            "On the web, the deck page's List opens rows, folds and toggles Art without lag in Firefox",
        ],
    },
    Release {
        version: Version::new(1, 19, 5),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web's deck page, Quick add ends where the widest chip row under it does",
            "Home's counters on the web have an Activity button, as in the app",
            "The Z in the web's nav is the same size as on zwipe.net",
            "Opening a card's row, or folding all, on the web's deck page List answers about twice as fast",
            "On the web's deck page, cards ease in, out and into new groups when a board is shown or hidden or the grouping changes",
            "On the web's deck page, switching to Stacks deals the cards in",
            "A card whose image is missing or fails to load shows its name, text and chips in the card's own shape, everywhere a card image shows",
            "The app's deck cards screen has a List and Stacks switch, Stacks showing two stacks side by side, a first tap revealing a card and a second opening its details",
        ],
    },
    Release {
        version: Version::new(1, 19, 4),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "Suggested commanders and the commander swipe on the web end with the site's footer, as Add does",
            "On the web, Playstyle and Activity take p, i, v and d on a card",
            "A staple's deck count on Playstyle opens to the decks that run it, each one tap from its deck",
            "On the web, an Oracle tag's examples take h, j, d and v, ? opens the hint of the open modal, and a card slides clear of its details where it fits",
            "On the web's deck page, Art grays out while Stacks is on, since stacks always show art",
            "Buying from TCGplayer adds each card in the printing you picked",
        ],
    },
    Release {
        version: Version::new(1, 19, 3),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "The Share dialog's link is a colored chip that wraps only when the screen is too narrow for it",
            "Charts grow in when their page opens or their section expands",
            "Printings has no Select where there is nothing to save the pick to",
            "The web's large dialogs fade and rise in when they open",
            "Buying is one cart on the web and in the app, on the deck's dock and the featured card, opening TCGplayer or Card Kingdom",
        ],
    },
    Release {
        version: Version::new(1, 19, 2),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "The deck page on the web deals its featured cards in, as a shared deck does",
            "Back returns to the screen you came from instead of looping between Create deck and Decks",
            "On the web, a cloned deck shows its own name as soon as it opens, and back and forward between two decks show the right one",
            "The Share dialog on the web shows the deck's own name in its link",
            "A melded card's list row shows the melded card in its thumbnail and hover preview",
            "Hovering a stack on the web's deck page no longer shifts its cards by a pixel",
        ],
    },
    Release {
        version: Version::new(1, 19, 1),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "On the web, u and Ctrl+Z undo on the deck page and the swipe screens",
            "On the web's deck page, d opens or closes a card's details",
            "Flipping a card on the web's deck page turns its stack image and row preview too",
            "A meld card's open row in a deck's list has the Meld toggle its details drawer has",
            "On the web, f opens the filter on Commander maybeboard, and a meld card's Meld is one switch for its image, details, row and stack card",
            "Image in a stack card's details on the web's deck page",
        ],
    },
    Release {
        version: Version::new(1, 19, 0),
        date: NaiveDate::from_ymd_opt(2026, 11, 13),
        entries: &[
            "The deck page on the web can show its cards as stacks, one per group, beside the list",
            "The web's deck page runs the window's width, its list in two columns on a wide screen",
            "A cart on each card group of the web's deck page, in the list and in the stacks, buys that group's cards",
            "Keyboard shortcuts on the web's deck page for the hovered or open card, with Shift+H to list them",
            "A shared deck's link names its owner and the deck",
            "A shared deck shows who shared it under its name",
        ],
    },
    Release {
        version: Version::new(1, 18, 17),
        date: NaiveDate::from_ymd_opt(2026, 11, 11),
        entries: &[
            "Add says it couldn't load cards and offers Retry when a search never answers",
            "Format, Deck tags and Oracle tags show more chips, with a hint box that hugs its text until you pick and, on the web, chips that scroll under the checkmark",
            "Printings tries a failed card image again once the connection is back",
            "A failure that repeats shows one error toast instead of a stack, and on the web Printings keeps Couldn't load printings and Retry clear of the toast",
            "Tapping a link or button on zwipe.net no longer flashes a blue highlight",
            "Links on zwipe.net and app.zwipe.net are the accent color with no underline, brightening slightly under a mouse",
            "Meld, a button on a meld card and in its details that shows the card it melds into",
            "Keyboard shortcuts on the web, with the arrows or hjkl to swipe and Shift+H to list them",
            "On a narrower web window, Add and Remove show the last swipe's chip at the top left under the nav",
            "A swiped card flies off again on Add, and the card's ghost sits exactly under it",
        ],
    },
    Release {
        version: Version::new(1, 18, 16),
        date: NaiveDate::from_ymd_opt(2026, 11, 9),
        entries: &[
            "Account's refresh buttons have a name for screen readers",
            "Decks shows each deck's name and tags together beside its art",
            "Account shows a pending email change right after you log in, and closing a hint no longer hides it",
            "Add no longer flashes another card before a new pile's top card shows",
            "The changelog shows which release is on the web and which is in the app stores",
            "Printings shows each card's outline while its image loads",
            "Card details' border shows on every edge and reads on dark themes",
            "Account's Pending row has Cancel, which drops a pending email change",
            "Expanded and hovered card rows keep their mana pips' outlines sharp",
            "On the web every button, link, field and row answers the mouse, disabled ones aside",
            "A failed request always says something went wrong instead of showing an empty error",
            "Distributions, Mana and Draw odds leave room above and below their charts",
            "Undoing a remove brings the card back with its MVP star",
            "Printings says when it couldn't load and has Retry, instead of sitting empty on a slow network",
            "The deck page on the web shows the deck's name over its chips",
            "Changelog releases look the same on the site and in the app, with the tag at the right end and no fill on hover",
            "Links inside text on the web answer the mouse too",
        ],
    },
    Release {
        version: Version::new(1, 18, 15),
        date: NaiveDate::from_ymd_opt(2026, 11, 9),
        entries: &[
            "Decks shows each deck's total price after its card count",
            "On iPhone the open menu no longer flickers when the theme changes",
            "A pending email has Resend and refresh buttons, so a lost link can be sent again",
            "Side drawers on the web stop above the footer instead of covering it",
            "A pending email's link expiry on the web is a tag, as in the app",
        ],
    },
    Release {
        version: Version::new(1, 18, 14),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "The web app no longer flashes unstyled on a hard refresh",
            "The web header folds behind the menu below 1110px wide instead of wrapping",
            "Toasts on the web rest just above the footer",
            "A toast over an open sheet on the web stays at the sheet's foot",
            "Back from a deck on the web returns to the screen that opened it",
            "Browser Back closes an open sheet before asking to leave",
            "Leave without saving? on the web keeps working after a visit to About",
            "Opening a web app link while logged out takes you there after you log in",
            "The web app's changelog shows a new release right after it ships",
            "Decks on the web folds deck tags behind Tags at every width",
            "On a wide window Home's stat chips sit right under the counters",
            "The title tag on the web's swipe screens has the same outline as the other label tags",
            "Changelog releases fold open and shut from their header, with Collapse all",
            "A shared deck link previews as a picture of the deck in its owner's theme",
            "Shared deck links show their preview on X and other apps that unfurl links",
            "zwipe.net's About shows the web app in its architecture",
            "Keyboard: every control is reachable with Tab and opens with Enter or Space",
            "Keyboard: zwipe.net has a skip link, a focus ring in every theme and a menu that Escape closes",
            "Screen readers: labels and headings name what's on screen, in order",
        ],
    },
    Release {
        version: Version::new(1, 18, 13),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "Sort on Decks, by name, size, format, price or last changed",
            "Home's Decks panel has a Create deck button beside All decks",
            "Long deck names wrap beside the art on Decks",
            "A folded group on Decks lights its whole border on hover",
            "Deck totals say how many cards have no price",
            "Sorting by price puts cards with no price last",
            "Partner search and the Partner pile offer only partners your commander can pair with",
            "Picking a commander with Partner with fills in its partner",
            "An Archidekt import fills in an empty deck's format and commander",
            "Import's Imported and Failed lists scroll in a window of their own",
            "Deck name, username and email fields check what you type as you type",
            "Save stays off while a field shows an error",
            "Quick add lists an exact name match first",
            "Quick add on the web lists its results under the box and closes them on a pick",
            "Typeahead results stand out from the panel behind them",
            "Every panel and section title has the same style",
            "Playstyle's Top color and Top colors rows count decks and say so",
            "Playstyle ranks only Oracle tags that appear in two or more decks",
        ],
    },
    Release {
        version: Version::new(1, 18, 12),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "Buttons, links and menu items read clearly in every light theme",
            "Coming back to the app refreshes your session, Home's figures and your settings",
            "Your latest Light or Dark choice stays put after the app opens, with no flip back",
            "The counters on Home and Log in read Live only while their copy is under five minutes old, otherwise as of its time",
            "The Pips setting's options read Themed and Classic",
            "You stay logged in on up to ten devices and browsers at once, up from five",
            "A setting changed just before the app closed is saved the next time it opens, not lost",
            "Logging out ends only that device's session",
            "The app downloads compressed data, so screens load faster on a slow connection",
            "A used or expired password reset link says so as soon as it opens",
            "Password managers fill in Log in, Create account and the password reset form",
        ],
    },
    Release {
        version: Version::new(1, 18, 11),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "Quick add on a deck's cards and on the Commander maybeboard clears with the same × as the filter's fields",
            "About's changelog shows five releases at a time, with Show more for the rest",
            "Add says Synergy on or Synergy off when you tap the bolt",
            "A swipe card's shadow shows all the way around, no longer cut off at its top and bottom",
            "Loading placeholders match the screen they stand in for, with every chip drawn empty",
            "A deck with one card type draws its Types chart as a single bar under the type's full name",
            "Every list of cards folds from its header, the Commander group and Suggested commanders included",
            "Swipe screens confirm each swipe in a quiet chip at the top left, with a Swipe confirmations setting for toasts or nothing instead",
            "On a wide window the deck page's card count, price and charts follow each card you add, remove, move or undo, with no reload",
            "Closing the Format, Deck tags or Oracle tags picker with changed picks asks Keep editing or Discard first",
            "Closing the Format, Deck tags or Oracle tags picker empties its search",
            "Undo on Add still steps back after you change the filter",
            "A card you add or move to the maybeboard and then undo comes back in Add",
            "A changed Config setting counts as a filter on its own, with No cards match when nothing fits",
            "A format picked in Is legal in beside the deck's own counts as a filter on Add",
            "Stack refreshed shows only when the refresh worked",
            "An empty swipe pile says it's empty in place of a blank card",
            "On a wide window Add's card stays clear of the card details panel",
            "Add on the web shows Couldn't load cards with Retry when it can't reach the server",
        ],
    },
    Release {
        version: Version::new(1, 18, 10),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "Keywords, card roles and Oracle tags use the app's regular chips, each group named by a label chip",
            "The error under a field on the deck, Log in and account forms starts with a capital letter",
            "Empty lists and empty Activity panels say so in a centered gray chip",
            "The card filter's Oracle tags hint explains include, exclude and the Dictionary",
            "The Format, Deck tags and Oracle tags pickers' ? hints match the website's",
            "The deck page's tag hints and the Oathbreaker field's hint are written for where they open, and other hints read tighter",
            "Enter closes a picker's ? hint on the web, and the hint no longer reopens on the next Enter",
            "Toasts, hints and errors use one name each for Oracle tags, the mainboard, Log in and Create account",
            "Charts name each card role in full, and a bad email address, deck name or failed undo says so in plain words",
            "Import errors you can fix read in plain words",
            "An email field says whether the address is missing or not valid",
            "Delete deck's dialog names the deck",
            "A one-card deck reads 1 card",
        ],
    },
    Release {
        version: Version::new(1, 18, 9),
        date: NaiveDate::from_ymd_opt(2026, 11, 8),
        entries: &[
            "A deck's commander and MVPs show their names in chips beside their role",
            "Card details slide in from the right as a panel titled with the card and its cost, the prices included",
            "Card rows match the card details, with their colors, price and P/T in the row",
            "An expanded card shows its own details first, with what the screen adds to it in its own section at the bottom",
            "A card's price and the deck's total follow the printing you chose, with no price when that printing has none",
            "Hovering a card row shows one preview at a time, gone once the pointer leaves the row",
            "A card's P/T reads as clearly as its price in every light theme",
            "Card details while swiping float in from the right as one card, clear of the header and the dock",
            "Printings show as a cover flow, the neighbors at the sides, stepped by the arrows or a swipe",
        ],
    },
    Release {
        version: Version::new(1, 18, 8),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Commander maybeboard and Playstyle put Swipe in their list's header, float Filter at the bottom right, and keep Clear in a Danger zone panel",
            "Could not load lines are muted chips, the same as other empty results",
            "The privacy policy no longer repeats the Fan Content notice",
            "Error messages start with a capital letter",
            "Playstyle's Oracle tags table shows long tags in full",
            "Playstyle with no decks says No playstyle yet",
        ],
    },
    Release {
        version: Version::new(1, 18, 7),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "zwipe.net's theme menu stays open after a pick",
            "zwipe.net's nav shows the stores as Apple and Play icons and its footer holds Contribute",
            "Hints and the guides on zwipe.net show an icon button as the icon itself instead of naming it in words",
            "zwipe.net's shared deck page shows the featured cards under the deck name, the cards in groups and the stats folded",
            "zwipe.net's theme picker opens as a drawer at the side on a wide window",
            "zwipe.net's changelog shows each release in a panel of its own",
            "zwipe.net's menu shows every link down to the bottom of the window, with no scrollbar",
            "zwipe.net's changelog shows five releases at a time, the rest behind Show more",
        ],
    },
    Release {
        version: Version::new(1, 18, 6),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Signed out, the main menu's palette and sun or moon change the theme on this device until you log in",
            "The Themes sheet saves a theme as you tap it and stays open, with no checkmark",
            "The App icon and Universes Beyond sheets save from the dock's checkmark, with no footer",
            "Config is now called Settings",
            "The Natural setting for mana symbols is now called Classic",
            "Light themes keep disabled chips, empty heatmap days and empty notes readable",
            "Settings' Theme row shows the palette in the theme's accent",
            "Settings' Change button for Theme carries the palette",
            "The Themes sheet groups its themes under chips",
        ],
    },
    Release {
        version: Version::new(1, 18, 5),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Account ends with a Danger zone panel for Log out and Delete account",
            "Account's Change username, email and password sheets save from a checkmark at the bottom right",
            "Change password has a Forgot? button in its header that emails you a reset link",
            "The password reset screen is titled Forgot password, as the menu names it",
            "Login, Create profile and Forgot password hold their fields in a panel headed with the screen's name, the featured flavor card under it",
            "Log in, Create profile and Send are full-width buttons under their forms, with chips to the other sign-in screens beneath",
            "Create profile and Forgot password return you to Log in once they succeed, with your username filled in",
            "Login opens scrolled to the top",
            "Change username and Change email refuse your current username or email",
            "The email change link's expiry shows as a chip",
        ],
    },
    Release {
        version: Version::new(1, 18, 4),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Create and Edit deck, the pickers and the filter and printing sheets save from a checkmark at the bottom right",
            "Create deck and Edit deck group their fields in Profile, Budget and Tags panels",
            "The tag pickers and the dictionary scroll to the screen's edge, with Search as a chip over the field",
            "The deck form's and the filter's fields clear from a × beside the box, which makes room for it as you type",
            "The Commander hint says a lit Filter keeps search to cards that can lead the format, and off lets any card lead",
            "The filter's Config clear puts every toggle back to its default, so the section reads as unset",
            "The land target stops at 0",
            "The filter sheet resets from an eraser",
            "Import calls the lines it could not add Failed",
            "Import and Export hold the decklist and Import's results in titled panels",
        ],
    },
    Release {
        version: Version::new(1, 18, 3),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Add, Remove, swipe select and Oracle tag examples float their filter, refresh and card details as icons at the bottom right",
            "Swipe screens give the card more room: the pile chips sit at the bottom beside the icons, and only the card's prices show under it",
            "Card details from the eye include the release date and artist",
            "Synergy on Add is a lightning bolt beside the filter",
            "Add switches between Search and Maybe with one chip after From",
            "Add deck cards, Remove deck cards and Deck cards are titled Add, Remove and Cards",
            "The card details sheet shows the details on the sheet itself, with no panel under them",
            "The card details sheet's Flip and Printings are icons in its dock at the bottom right",
            "When a swipe pile runs out, a toast says so",
        ],
    },
    Release {
        version: Version::new(1, 18, 2),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "The deck's card list has a floating bar of icons for add, remove, filter and undo; an x on the filter resets it",
            "Card rows show the whole name, with power and toughness, price and colors beside it or wrapped underneath",
            "The color map is gone: Home, Playstyle and a deck's Mana section show colors as bars with mana pips",
            "Distributions shows the top five card roles and a new oracle tags chart, the rest behind See more",
            "Draw odds shows its top five groups, the rest behind See more",
            "Draw odds names the opening hand turn 0",
            "A fold-all button after the Show chips shuts every card group, and opens them again",
            "A deck's Colors chart counts a multicolor card under each of its colors",
            "Mana cost fulfillment shows each color's pip, with its bar in that color",
        ],
    },
    Release {
        version: Version::new(1, 18, 1),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Decks creates a deck from a + at the bottom right",
            "The deck view's actions sit on the page: Edit, Clone and Share as icons by the profile, the store links in Budget, and Import, Export, Clear skips and Delete in Data and Danger zone panels at the bottom",
            "Home shows your three latest decks as tiles with a button to all of them",
            "Decks lists your decks one to a row with each deck's art beside its name, with Group by folding them under chips",
            "Decks spaces every row evenly, from the chip rows to the decks",
            "The last deck in a group keeps its bottom border",
            "Back after deleting a deck returns to Decks, not the deleted deck",
            "A one-card deck shows 1 card in the deck list",
        ],
    },
    Release {
        version: Version::new(1, 18, 0),
        date: NaiveDate::from_ymd_opt(2026, 11, 5),
        entries: &[
            "Main menu behind a hamburger in every screen's header: the deck screens as a list, then Home, Account, Settings, About, a palette for the theme and a sun or moon for Dark/Light as icons, with Help & feedback and Log out",
            "Back is an arrow at the top left of every screen",
            "The header slides away as you scroll down, its title and buttons fading into it, and comes back as you scroll up",
            "No screen has a bottom bar; what each screen does floats as icons at its bottom right",
            "Sheets close from an x in their corner or a drag down",
            "Floating buttons rise above the keyboard",
            "About, a new screen in the main menu with the version, links, credits and the changelog, each release in a panel of its own",
            "About and the privacy policy open from the main menu signed out",
            "The main menu and the sheets drop the grid of squares behind their contents",
        ],
    },
    Release {
        version: Version::new(1, 17, 0),
        date: NaiveDate::from_ymd_opt(2026, 11, 1),
        entries: &[
            "Card roles with no Oracle tags to open show as grey chips",
            "Activity's Card roles, Oracle tags and Most skipped say what fills them while they're empty",
            "Activity's role and tag charts say why they're empty for the pick under Show",
            "Activity's hints say share instead of rate",
            "Links open inside Zwipe instead of switching to the browser",
            "Hints with a guide on zwipe.net link to it",
        ],
    },
    Release {
        version: Version::new(1, 16, 2),
        date: NaiveDate::from_ymd_opt(2026, 10, 29),
        entries: &[
            "Activity's Oracle tags chart lists its tags highest share first",
            "Activity's Oracle tags chart shows up to ten tags where your rate beats other players'",
            "Activity's counts roll until the rest of the screen loads, then land with it",
        ],
    },
    Release {
        version: Version::new(1, 16, 1),
        date: NaiveDate::from_ymd_opt(2026, 10, 28),
        entries: &[
            "The color map's key sits under a rule, the way a panel's buttons do",
            "Featured flavor's card name, price and buttons sit under a rule",
            "Activity shows its tags and Totals as ghosts while it loads, keeping the grid's space empty",
            "Activity opens with your own counts: decks created, searches run and cards swiped",
            "Config names the theme's mode Dark or Light, capitalized",
        ],
    },
    Release {
        version: Version::new(1, 16, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 27),
        entries: &[
            "An Activity screen, opened from Home's counters, maps your swipes and searches over the last year",
            "Activity shows what your swipes came to and your rates by card role and oracle tag",
            "Activity lists your most skipped cards, the ones you skip and never keep",
            "Home's Playstyle button sits at the end of the color map's key",
            "Featured flavor's Image and Buy are buttons beside its card name and price",
            "Home's counts switch between everybody and only you",
            "Chart titles are chips centered over their charts, with their ? beside them",
            "Playstyle with no decks shows Home's line, Build a deck to see your playstyle",
        ],
    },
    Release {
        version: Version::new(1, 15, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 27),
        entries: &[
            "Skipped cards return to a deck's swipe pile after a week, removed cards after a month",
            "Returning skipped cards come after cards you haven't seen",
            "Synergy moves on to related cards once your commander's picks run out",
            "The changelog's version chips group patches under their minor version",
            "Swiping to add cards shows every recommended card before running out",
            "Card details open in a bottom sheet",
            "Playstyle, a new screen with your color map, card roles, staples and suggested commanders",
            "Home shows your color map",
            "Profile splits into Account and Config",
            "Log out moves to Account",
            "Draw odds' ? sits in its section header",
            "The deck view's Roles chart reveals each role's tags",
            "The deck charts are titled Types, Roles and Colors, with Colors under Mana",
            "Mana symbols follow your theme's colors, with a Config setting to keep the classic ones",
        ],
    },
    Release {
        version: Version::new(1, 14, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 23),
        entries: &[
            "Profile's Pending row shows when the confirmation link expires",
            "The changelog narrows by year, month and version",
            "Field labels with a ? sit centered with it, with more room above the field",
            "Delete account's password field sits a line below the warning",
        ],
    },
    Release {
        version: Version::new(1, 13, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 22),
        entries: &[
            "Registering sends a verification email and takes you to log in",
            "Changing your email takes effect once you confirm it from the new address",
            "Passwords do not match waits until you finish typing the confirmation",
        ],
    },
    Release {
        version: Version::new(1, 12, 2),
        date: NaiveDate::from_ymd_opt(2026, 10, 21),
        entries: &[
            "Flexoki theme replaces Visual Studio, whose users move to VS Code",
            "Synthwave '84's colors match the original theme",
        ],
    },
    Release {
        version: Version::new(1, 12, 1),
        date: NaiveDate::from_ymd_opt(2026, 10, 20),
        entries: &[
            "Decks shows the Tags chip while loading instead of a row of placeholder tags",
            "iOS: the app icon can match your theme, switching with it in dark and light",
            "The Zwipe logo builds left to right, its shape arriving as noise and then decoding, on the splash and the home screen",
            "The splash's logo leaves the way it arrived, turning back to noise and clearing left to right before the home screen shows",
            "The Z's second row reaches one column further left, in the logo and on the app icons",
        ],
    },
    Release {
        version: Version::new(1, 12, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 19),
        entries: &[
            "Four Monokai Pro themes: Pro, Machine, Octagon and Spectrum, in dark and light",
            "Editor defaults as themes: Darcula, Xcode, Visual Studio, Eclipse and Neovim, in dark and light",
            "Ten more themes: Bluloco, GitHub High Contrast, Kanagawa Dragon, Material, Melange, Modus, Nightfox, Oxocarbon, Poimandres and Terafox, in dark and light",
            "The theme and App icon pickers group themes under headings by look, with high contrast and color blind last",
            "Ristretto is now called Monokai Pro Ristretto",
            "iOS: the app icon on Profile comes in the new themes too",
            "The backdrop is a field of shaded squares instead of grid lines, on screens and sheets",
            "The app opens on a short splash: the backdrop rains in and the mark resolves over it before the first screen, whose figures roll as it leaves",
            "Chip row labels are tags with no colon",
            "The deck list's tags fold behind a Tags chip",
            "The Oracle tags dictionary's letter rail fades only at the edge with more letters past it",
            "The filter sheet's sections sit closer together",
        ],
    },
    Release {
        version: Version::new(1, 11, 6),
        date: NaiveDate::from_ymd_opt(2026, 10, 18),
        entries: &[
            "Hint text no longer fades out at the edges when it all fits",
            "iOS: the app icon on Profile can be drawn in any theme's colors, dark or light",
        ],
    },
    Release {
        version: Version::new(1, 11, 5),
        date: NaiveDate::from_ymd_opt(2026, 10, 16),
        entries: &[
            "Reset password shows its outcome in a panel",
            "Dialogs fade out when they close",
            "Disabled dialog buttons look disabled",
        ],
    },
    Release {
        version: Version::new(1, 11, 4),
        date: NaiveDate::from_ymd_opt(2026, 10, 11),
        entries: &[
            "Switching dark mode sweeps the new theme across the screen again instead of cutting to it",
        ],
    },
    Release {
        version: Version::new(1, 11, 3),
        date: NaiveDate::from_ymd_opt(2026, 10, 11),
        entries: &[
            "Home counters show the last numbers fetched when there is no connection",
            "Home no longer shows an error when its counters cannot be refreshed",
            "Messages show two lines until tapped, then the whole text",
            "Long messages wrap instead of running off the screen",
            "Requests give up after thirty seconds instead of waiting forever",
            "Changing your email no longer writes the new address to the device log",
            "Android: the sign-in session is left out of device backups",
            "Too many wrong passwords lock out only the device that sent them",
        ],
    },
    Release {
        version: Version::new(1, 11, 2),
        date: NaiveDate::from_ymd_opt(2026, 10, 9),
        entries: &[
            "Sessions renew a couple of minutes before they expire",
            "Logging out ends every device's session within minutes",
        ],
    },
    Release {
        version: Version::new(1, 11, 1),
        date: NaiveDate::from_ymd_opt(2026, 10, 9),
        entries: &["The light and dark switch moves into the Theme row on Profile"],
    },
    Release {
        version: Version::new(1, 11, 0),
        date: NaiveDate::from_ymd_opt(2026, 10, 9),
        entries: &[
            "A ? beside the new password field lists the password rules",
            "Privacy policy says where data is stored and how to request a copy",
            "Numbers, names and links each have their own color in every theme",
            "Titles, selections and labels use the colors each theme's own editor scheme gives them",
            "Theme picker swatches show each theme's six palette colors",
            "Verification and password reset emails show their link as plain text",
            "The chip under the home counters says all users",
            "Screen titles use one capitalization style",
            "Panels sit the same distance apart on every screen",
        ],
    },
    Release {
        version: Version::new(1, 10, 7),
        date: NaiveDate::from_ymd_opt(2026, 10, 6),
        entries: &[
            "iOS: buttons respond while a list is still scrolling",
            "iOS: swiping back from the screen edge works while a list is still scrolling",
        ],
    },
    Release {
        version: Version::new(1, 10, 6),
        date: NaiveDate::from_ymd_opt(2026, 10, 5),
        entries: &["Changing the theme wipes the new one across the screen"],
    },
    Release {
        version: Version::new(1, 10, 5),
        date: NaiveDate::from_ymd_opt(2026, 10, 3),
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
        version: Version::new(1, 10, 4),
        date: NaiveDate::from_ymd_opt(2026, 10, 2),
        entries: &[
            "Deck cards: the commander and MVP row waits for every card image, then opens and deals the cards in",
            "Light themes draw the logo in the theme's primary accent",
            "The logo on the home and sign-in screens resolves from static as the screen opens",
        ],
    },
    Release {
        version: Version::new(1, 10, 3),
        date: NaiveDate::from_ymd_opt(2026, 9, 30),
        entries: &[
            "Messages appear at the top left, over the title bar, clear of the buttons",
            "Several messages at once stack; tap to open, tap again to close",
            "Messages stay up longer",
            "Save on the Themes, Exceptions, Deck tags, Oracle tags, Format, Filter and Printing sheets is grayed out until something changes; Back puts things back",
            "Swiping back out of the Oracle tags picker drops your edits, the same as Back",
            "Dark mode and Universes Beyond say what they switched to",
            "Every deck form field and most Profile rows have their own ?",
            "Shorter hints across the app",
            "Commander select no longer offers cards that are only a legendary creature on their back face, or melded cards; a deck that already has one warns",
            "Oathbreaker signature spell select no longer offers Adventure creatures; split cards still qualify",
        ],
    },
    Release {
        version: Version::new(1, 10, 2),
        date: NaiveDate::from_ymd_opt(2026, 9, 22),
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
        version: Version::new(1, 10, 1),
        date: NaiveDate::from_ymd_opt(2026, 9, 6),
        entries: &[
            "Every failure while talking to the server says so, with the same brief message on every screen",
            "A failed card batch while swiping says so instead of the pile running dry",
            "The Decks screen and the commander maybeboard show a note chip when their lists can't load",
            "Links on zwipe.net open in the same tab; Ctrl or middle click still opens a new one",
        ],
    },
    Release {
        version: Version::new(1, 10, 0),
        date: NaiveDate::from_ymd_opt(2026, 9, 1),
        entries: &[
            "A Universes Beyond setting in Profile hides crossover cards from searches and commander picks, with per-franchise exceptions",
            "Set filters check every printing of a card",
            "Searches show a card's in-universe printing when it has one",
            "An empty search says why",
        ],
    },
    Release {
        version: Version::new(1, 9, 3),
        date: NaiveDate::from_ymd_opt(2026, 8, 20),
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
        version: Version::new(1, 9, 2),
        date: NaiveDate::from_ymd_opt(2026, 8, 18),
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
        version: Version::new(1, 9, 1),
        date: NaiveDate::from_ymd_opt(2026, 8, 14),
        entries: &[
            "Keyword reminders for the newest sets explain the mechanic instead of pointing at the card",
            "Keyword definitions update on their own, no app update needed",
            "Tapping a card role tag on a commander maybeboard entry shows its description and Examples",
            "Evened out the spacing on the deck list console and the deck profile",
        ],
    },
    Release {
        version: Version::new(1, 9, 0),
        date: NaiveDate::from_ymd_opt(2026, 8, 13),
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
        version: Version::new(1, 8, 1),
        date: NaiveDate::from_ymd_opt(2026, 8, 12),
        entries: &[
            "Tap a group header on your deck's card list to collapse it",
            "Search finds cards whose newer foreign printing was hiding the English one",
            "Loading screens for the deck list and deck cards match the real layouts",
            "The commander and MVP cards at the top of the deck deal in from above",
            "Deck list group headers drop their underline",
        ],
    },
    Release {
        version: Version::new(1, 8, 0),
        date: NaiveDate::from_ymd_opt(2026, 8, 12),
        entries: &[
            "The deck list has Group by and Show rows",
            "A one-time tip on the deck list covers the new rows; ? brings it back",
            "The import and export screens keep their controls in place while results scroll underneath",
            "Bolder chip row labels",
        ],
    },
    Release {
        version: Version::new(1, 7, 6),
        date: NaiveDate::from_ymd_opt(2026, 8, 10),
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
        version: Version::new(1, 7, 5),
        date: NaiveDate::from_ymd_opt(2026, 8, 4),
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
        version: Version::new(1, 7, 4),
        date: NaiveDate::from_ymd_opt(2026, 7, 30),
        entries: &[
            "Fixed a crash when saving a card image to your photos from a long press",
            "Anonymous error and crash reports, with no account or personal data attached",
        ],
    },
    Release {
        version: Version::new(1, 7, 3),
        date: NaiveDate::from_ymd_opt(2026, 7, 24),
        entries: &[
            "Filter edits wait for Apply; Reset, Cancel and tapping outside each say what they did",
            "Average power and toughness in the Distributions section",
            "Oracle tag search puts exact matches first and searches descriptions",
            "Shared deck pages keep card groups in order on phones",
            "Tap an oracle tag in a card's details to read its definition while you swipe",
        ],
    },
    Release {
        version: Version::new(1, 7, 2),
        date: NaiveDate::from_ymd_opt(2026, 7, 20),
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
        version: Version::new(1, 7, 1),
        date: NaiveDate::from_ymd_opt(2026, 7, 17),
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
        version: Version::new(1, 7, 0),
        date: NaiveDate::from_ymd_opt(2026, 7, 14),
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
        version: Version::new(1, 6, 0),
        date: NaiveDate::from_ymd_opt(2026, 7, 12),
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
        version: Version::new(1, 5, 0),
        date: NaiveDate::from_ymd_opt(2026, 7, 10),
        entries: &[
            "Swipe back from the left edge to return to the previous page",
            "Every screen keeps its own filters per deck",
            "An expanded card stays highlighted so you keep your place",
            "Smoother loading screens; a filter no longer carries over between screens",
        ],
    },
    Release {
        version: Version::new(1, 4, 0),
        date: NaiveDate::from_ymd_opt(2026, 7, 8),
        entries: &[
            "The commander picker leads with the community's most-built commanders, in a fresh order each day",
            "Partners that name each other pair automatically",
            "Star up to three MVP cards per deck",
            "Share a deck with a public link from its More menu",
        ],
    },
    Release {
        version: Version::new(1, 3, 1),
        date: NaiveDate::from_ymd_opt(2026, 7, 3),
        entries: &[
            "Anonymous diagnostics, with no account or gameplay data involved",
            "Internal cleanups for faster, more stable swiping",
        ],
    },
    Release {
        version: Version::new(1, 3, 0),
        date: NaiveDate::from_ymd_opt(2026, 7, 2),
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
        version: Version::new(1, 2, 1),
        date: NaiveDate::from_ymd_opt(2026, 7, 1),
        entries: &[
            "Tap the eye button while swiping to read a card's full rules and stats",
            "The app opens straight into its themed layout",
        ],
    },
    Release {
        version: Version::new(1, 2, 0),
        date: NaiveDate::from_ymd_opt(2026, 6, 30),
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
        version: Version::new(1, 1, 4),
        date: NaiveDate::from_ymd_opt(2026, 6, 29),
        entries: &[
            "Set a land target, your own or the format default, with a heads-up if you drop below it",
            "Filter cards by price in USD, EUR or Tix",
            "Set a deck price target with alerts as your total approaches and passes it",
            "Deck Stats, Distributions, Mana and Warnings are tap-to-expand sections",
            "More consistent filter controls; fixed a flicker on startup",
        ],
    },
    Release {
        version: Version::new(1, 1, 3),
        date: NaiveDate::from_ymd_opt(2026, 6, 28),
        entries: &[
            "Card names show while you swipe, including alternate art and non-English printings",
            "Tap a field to set your format, commander or tags, with your deck name checked as you type",
            "More strategy tags with definitions, plus format and power-level pickers",
            "Read the Privacy Policy in-app, from Profile",
            "Sign-in, sign-up and profile forms show errors under each field",
        ],
    },
    Release {
        version: Version::new(1, 1, 2),
        date: NaiveDate::from_ymd_opt(2026, 6, 27),
        entries: &["Filter controls match across the card-swipe screens"],
    },
    Release {
        version: Version::new(1, 1, 1),
        date: NaiveDate::from_ymd_opt(2026, 6, 26),
        entries: &[
            "Every screen has a Help button to report a problem or open our Discord",
            "Quick tips on the Import and Export screens",
            "Bug fixes and reliability improvements",
        ],
    },
    Release {
        version: Version::new(1, 1, 0),
        date: NaiveDate::from_ymd_opt(2026, 6, 24),
        entries: &[
            "Add up to 5 strategy tags, shown on the deck list and deck page",
            "A Zwipe option swipes through legal commanders, partners, backgrounds and signature spells instead of searching",
            "Tap any deck card to expand it, with mana cost, type, rules text and power/toughness or loyalty",
            "Tap a keyword for a plain-language reminder; a Keywords button lists every keyword on the card",
            "Drop shadows on mana symbols, roomier pips, smoother card details and scrollable dialogs",
        ],
    },
    Release {
        version: Version::new(1, 0, 10),
        date: NaiveDate::from_ymd_opt(2026, 6, 23),
        entries: &[
            "Commander search shows a Searching hint while results load",
            "Buy links show an arrow to signal they open an external store",
            "A clearer update-required screen",
            "Small under-the-hood polish",
        ],
    },
    Release {
        version: Version::new(1, 0, 9),
        date: NaiveDate::from_ymd_opt(2026, 6, 15),
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
        version: Version::new(1, 0, 7),
        date: NaiveDate::from_ymd_opt(2026, 6, 12),
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
        version: Version::new(1, 0, 6),
        date: NaiveDate::from_ymd_opt(2026, 6, 11),
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
        version: Version::new(1, 0, 2),
        date: NaiveDate::from_ymd_opt(2026, 6, 8),
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
        version: Version::new(1, 0, 1),
        date: NaiveDate::from_ymd_opt(2026, 6, 7),
        // Placeholder until the original release notes turn up.
        entries: &["Various performance improvements"],
    },
    Release {
        version: Version::new(1, 0, 0),
        date: NaiveDate::from_ymd_opt(2026, 6, 6),
        entries: &["Initial release"],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn every_release() -> impl Iterator<Item = &'static Release> {
        UPCOMING.iter().chain(RELEASES)
    }

    fn every_entry() -> impl Iterator<Item = (Version, &'static str)> {
        every_release().flat_map(|release| {
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

    /// Every shipped release has a real date and nothing upcoming has one.
    #[test]
    fn only_shipped_releases_have_dates() {
        let undated: Vec<_> = RELEASES
            .iter()
            .filter(|release| release.date.is_none())
            .map(|release| release.version)
            .collect();
        assert!(undated.is_empty(), "{undated:#?}");
        assert!(UPCOMING.iter().all(|release| release.date.is_none()));
    }

    /// RELEASES is newest first; two releases may share a day.
    #[test]
    fn releases_are_newest_first_by_date() {
        let out_of_order: Vec<_> = RELEASES
            .windows(2)
            .filter_map(|pair| match pair {
                [newer, older] if newer.date < older.date => Some((newer.version, older.version)),
                _ => None,
            })
            .collect();
        assert!(out_of_order.is_empty(), "{out_of_order:#?}");
    }

    /// RELEASES is strictly newest first by version.
    #[test]
    fn releases_are_newest_first_by_version() {
        let out_of_order: Vec<_> = RELEASES
            .windows(2)
            .filter_map(|pair| match pair {
                [newer, older] if newer.version <= older.version => {
                    Some((newer.version, older.version))
                }
                _ => None,
            })
            .collect();
        assert!(out_of_order.is_empty(), "{out_of_order:#?}");
    }

    /// Every upcoming version is newer than the latest shipped one.
    #[test]
    fn upcoming_is_newer_than_the_latest_release() {
        let latest = RELEASES.first().map(|release| release.version);
        let stale: Vec<_> = UPCOMING
            .iter()
            .map(|release| release.version)
            .filter(|version| Some(*version) <= latest)
            .collect();
        assert!(stale.is_empty(), "{stale:#?}");
    }

    /// The workspace version is the latest release or an upcoming one, so a
    /// cut that misses the `Cargo.toml` bump or the changelog move fails.
    #[test]
    fn workspace_version_is_latest_or_upcoming() {
        let workspace: Version = env!("CARGO_PKG_VERSION")
            .parse()
            .expect("the workspace version parses");
        let known: Vec<_> = UPCOMING
            .iter()
            .chain(RELEASES.first())
            .map(|release| release.version)
            .collect();
        assert!(known.contains(&workspace), "{workspace} not in {known:?}");
    }

    /// No version appears twice across upcoming and shipped releases.
    #[test]
    fn all_versions_are_distinct() {
        let mut seen = HashSet::new();
        let mut repeats: Vec<_> = every_release()
            .map(|release| release.version)
            .filter(|version| !seen.insert(*version))
            .collect();
        repeats.sort_unstable();
        assert!(repeats.is_empty(), "{repeats:#?}");
    }

    /// An entry has words and no em dash.
    #[test]
    fn no_entry_is_empty_or_uses_an_em_dash() {
        let offenders: Vec<_> = every_entry()
            .filter(|(_, entry)| entry.trim().is_empty() || entry.contains('\u{2014}'))
            .collect();
        assert!(offenders.is_empty(), "{offenders:#?}");
    }

    #[test]
    fn the_mobile_store_version_names_a_shipped_release() {
        assert!(
            RELEASES.iter().any(|r| r.version == MOBILE_STORE_VERSION),
            "{MOBILE_STORE_VERSION} is in RELEASES"
        );
    }
}
