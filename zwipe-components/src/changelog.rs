//! Shared changelog.
//!
//! The release history, rendered identically on the website (`zite`) and in the
//! app (`zwiper`). Data and styling live here so the two surfaces never drift;
//! each consumer wraps the [`Changelog`] component in its own chrome (a page
//! with nav/footer on the web, a bottom sheet in the app). Styling is in
//! `assets/components.css`.

use dioxus::prelude::*;

/// One shipped version and its notes. Notes sourced from the App Store "What's
/// New" history, dates from the App Store Connect submission history (public
/// "Ready for Distribution" date); newest first. Em dashes recast for copy
/// style.
struct Release {
    version: &'static str,
    date: &'static str,
    entries: &'static [&'static str],
}

/// Versions in progress for the next release. Rendered at the top of the
/// changelog with an "Upcoming" badge instead of "Latest".
const UPCOMING: &[Release] = &[Release {
    version: "1.6.1",
    date: "Coming soon",
    entries: &[
        "Your deck's Lands count and target now sit in the Mana section next to average mana value, instead of under Budget.",
    ],
}];

const RELEASES: &[Release] = &[
    Release {
        version: "1.6.0",
        date: "Jul 12, 2026",
        entries: &[
            "In-app changelog. Browse every release without leaving the app, right from your Profile.",
            "A cleaner deck card screen with squircle mana pips, an inline price tag, and power/toughness on each row.",
            "Theme and dark-mode controls now live in your Profile.",
            "Seventeen new themes, thirty-one in all: editor classics like VS Code, GitHub, One Dark, Kanagawa, Ayu, and Night Owl, a warm-grey docs.rs look for the Rustaceans, plus bolder picks like Synthwave '84, Vantablack, PowerShell, and a green-on-black Hackerman mode.",
            "A new Achromatopsia theme for total color blindness joins the colorblind set, and every theme got a contrast pass so dimmed text stays easy to read.",
            "Your theme now sticks: the app and website open in your last-used theme, so the sign-in screens are already themed before you log in, and the website remembers your pick between visits.",
            "Buy a card straight from the home screen, tap its price for TCGplayer or Card Kingdom.",
            "Fixed importing double-faced cards like Boggart Trawler // Boggart Bog.",
            "A light-mode polish pass across every theme, cleaner panels and softer mana-pip shadows.",
            "Oracle tags, find and build with community-maintained tags for what cards actually do (removal, ramp, card advantage, tutors, and more).",
            "Card roles on every card: expand a card to see what it does, like removal, ramp, card advantage, or aggression, and tap a role to open the community oracle tags underneath.",
            "Give your deck a game plan: choose the oracle tags that describe your strategy, and picking an archetype like Aggro or Aristocrats seeds the matching ones for you.",
            "New in-app guides explain deck tags, card roles, and oracle tags, so it's clear what each one means and how they shape your suggestions.",
            "Deck color identity shows as mana pips right after each deck's name.",
            "Every mana pip now tints its outline and glyph to its own mana color, so the colors stay instantly recognizable in any theme.",
            "A reorganized deck view: Profile, Budget, and Tags are now tidy sections, your land and price targets read as clear goal-versus-actual rows, and the Budget section collapses to stay out of the way.",
            "Shared deck pages now show each card's roles and the deck's full price to the cent, command zone included.",
            "More polish: two-color hybrid mana symbols now sit centered in their pips, keyword and card-role sections are clearly labeled, and the card-detail reveals ease open and closed instead of snapping.",
        ],
    },
    Release {
        version: "1.5.0",
        date: "Jul 10, 2026",
        entries: &[
            "Swipe back: drag from the left edge of the screen to return to the previous page, the way you navigate everywhere else on your phone.",
            "Filters that remember: every screen now keeps its own filters for each deck, so switching decks or stepping away picks up right where you left off.",
            "Cleaner card browsing: an expanded card now stays highlighted so you keep your place while you look through your deck.",
            "Smoother loading screens and a fix for a filter that could sometimes carry over between screens, plus general polish.",
        ],
    },
    Release {
        version: "1.4.0",
        date: "Jul 8, 2026",
        entries: &[
            "Pick your commander by popularity: the commander picker now leads with the community's most-built commanders, in a fresh order each day, so you start from what people are actually building.",
            "Partners that name each other now pair automatically: choose one and Zwipe fills in its partner for you.",
            "Star your deck's MVPs: mark up to three standout cards in any deck so your key pieces stand out at a glance, on the deck list and while you build.",
            "Share your deck: send a public link to any deck straight from its More menu, and anyone can open it on the web to see your list.",
        ],
    },
    Release {
        version: "1.3.1",
        date: "Jul 3, 2026",
        entries: &[
            "Under-the-hood improvements: anonymous, PII-free app health signals help us spot and fix rough edges faster. No account or gameplay data involved.",
            "Small internal cleanups to keep swiping fast and stable.",
        ],
    },
    Release {
        version: "1.3.0",
        date: "Jul 2, 2026",
        entries: &[
            "Zwipe now remembers your skips: cards you swipe left on stay out of that deck's suggestions, saved the instant you swipe, even if you close the app right after. Cards you deliberately remove from a deck stay out too.",
            "Changed your mind? Clear a deck's skips anytime from its More menu.",
            "Every deck now remembers your place: leave the add screen, work on another deck, and come back to pick up right where you left off, undo history included.",
            "Reached the end of your results? Swipe down on the empty stack to step back through your last swipes.",
            "Cards you send to the maybeboard now show up there immediately.",
            "Your deck's card list now sorts alphabetically by default.",
            "See which version you're on at the bottom of the Profile screen.",
            "Privacy policy updated to describe how your activity personalizes suggestions.",
            "Extra polish: card images ease in as they load, and the card layout keeps consistent spacing on every screen size.",
        ],
    },
    Release {
        version: "1.2.1",
        date: "Jul 1, 2026",
        entries: &[
            "Tap the new eye button while swiping to read a card's full rules text and stats: rules with real mana symbols, type line, rarity, keywords, and power/toughness or loyalty. Great for alternate-art or textless printings that hide the details.",
            "Smoother startup: the app opens straight into its themed layout, with no quick flash of unstyled content.",
        ],
    },
    Release {
        version: "1.2.0",
        date: "Jun 30, 2026",
        entries: &[
            "See your deck's odds: a new draw-odds view shows your chance of drawing a land, ramp, removal, and more, in your opening hand or by any turn.",
            "Set a power level for your deck, and add descriptive tags like Budget, Jank, Meme, or Precon.",
            "Many more strategy tags to label your deck at a glance: dozens of popular themes, each with a plain-language definition.",
            "Toggle synergy suggestions on or off right on the add-cards screen as you build.",
            "Smarter building: contradictory include/exclude filters are caught before you apply them, and lands drop out of the add screen once you hit your land target.",
            "The create and edit deck screens now open at the top, plus leader-eligibility fixes and other polish.",
        ],
    },
    Release {
        version: "1.1.4",
        date: "Jun 29, 2026",
        entries: &[
            "Set a land target for your deck, pick your own number or use the format's default, and get a heads-up while you build if you drop below it.",
            "Build on a budget: filter cards by price (USD, EUR, or Tix) to find options in your range.",
            "Set a price target for a deck and get alerts as your total approaches and passes it.",
            "Your deck's Stats, Distributions, Mana, and Warnings are now grouped into tidy, tap-to-expand sections.",
            "Cleaner, more consistent filter controls, plus a fix for a flicker when the app first opens.",
        ],
    },
    Release {
        version: "1.1.3",
        date: "Jun 28, 2026",
        entries: &[
            "Card names now show while you swipe, so every printing is identifiable at a glance, including alternate art and non-English cards.",
            "A smoother deck builder: tap a field to set your format, commander, or tags; empty fields read \"Not set,\" and your deck name is checked as you type.",
            "An expanded set of strategy tags, each with a plain-language definition, plus format and power-level pickers.",
            "Read the Privacy Policy right inside the app, from your Profile.",
            "Clearer sign-in, sign-up, and profile forms: validation errors now appear directly under each field.",
        ],
    },
    Release {
        version: "1.1.2",
        date: "Jun 27, 2026",
        entries: &[
            "A small consistency fix to the card-swipe screens so the filter controls look and behave the same everywhere.",
        ],
    },
    Release {
        version: "1.1.1",
        date: "Jun 26, 2026",
        entries: &[
            "Get help without leaving the app: every screen now has a Help button, tap it to report a problem (it opens your email, pre-filled with your app version) or jump into our Discord community.",
            "New quick tips on the Import and Export screens walk you through importing a decklist or sharing your deck.",
            "Bug fixes and reliability improvements under the hood.",
        ],
    },
    Release {
        version: "1.1.0",
        date: "Jun 24, 2026",
        entries: &[
            "Tag your decks: add up to 5 strategy tags (Aggro, Tokens, Reanimator, and more) to label a deck at a glance, your tags show on your deck list and deck page.",
            "Swipe to pick your command zone: a new \"Zwipe\" option lets you swipe through legal choices for your commander, partner, background, or signature spell instead of typing a search.",
            "Tap any card in your deck to expand it, now with its mana cost, type, rules text rendered with real mana symbols, and power/toughness or starting loyalty.",
            "Keyword helper: tap a keyword to get a plain-language reminder of what it does, and a Keywords button while swiping lists every keyword on the current card.",
            "Polish: card-style drop shadows on mana symbols, roomier pips, smoother expanding card details, and dialogs that scroll when there's a lot to show.",
        ],
    },
    Release {
        version: "1.0.10",
        date: "Jun 23, 2026",
        entries: &[
            "Commander search now shows a \"Searching…\" hint while it looks, so a card never seems missing while results load.",
            "Buy links (TCGplayer, Card Kingdom) now show an arrow (↗) so it's clear a tap opens an external store.",
            "A cleaner, clearer \"update required\" screen for when a new version is needed.",
            "Small under-the-hood polish.",
        ],
    },
    Release {
        version: "1.0.9",
        date: "Jun 15, 2026",
        entries: &[
            "A brand-new app icon.",
            "Your deck list got a quick-read makeover: each deck's details now show as color-coded tags, and the card count turns yellow when a deck is the wrong size for its format, so you can spot an off-size deck at a glance.",
            "More accurate deck checks: Oathbreaker, Brawl, Historic Brawl, and Gladiator now use their correct legal deck sizes.",
            "Profile, reorganized: your account details and your Preferences (theme and dark mode) are now separate, clearly labeled cards. Tap Change on either to edit it in a quick slide-up sheet, and account deletion tucks neatly under a More menu.",
            "Cleaner deck view: the Profile, Stats, and Warnings panels, and the charts, now carry their titles inside each card.",
            "The home-screen quote holds steady as you move around the app instead of flickering, and refreshes itself over time.",
            "Small polish: card-mover buttons now read \"To mainboard,\" long deck names wrap instead of clipping, loading placeholders render solid, and a warmer tweak to the Gruvbox theme text.",
        ],
    },
    Release {
        version: "1.0.7",
        date: "Jun 12, 2026",
        entries: &[
            "A fresh look across the whole app: a subtle background grid and layered depth give every screen a cleaner, more tactile feel, with crisp readable panels floating above it.",
            "Rebuilt theme picker: every theme now previews its actual colors as little swatches, so you can see a palette before you choose it. Dark mode sits right on top and the colorblind-friendly themes are grouped together.",
            "Preferences now slides up as a sheet and previews live as you tap, so you can try themes without losing your place, Save to keep, back out to revert.",
            "Real mana symbols: filters and your deck's card list now show proper colored mana pips instead of letters.",
            "Tap a card's name on the home screen to pop open its full art.",
            "Cleaner sheets and dialogs everywhere, with consistent headings and full-width dividers.",
            "Lots of small polish: tidier inputs and chips, refined spacing, and clearer headers across home, sign-in, decks, import, and export.",
        ],
    },
    Release {
        version: "1.0.6",
        date: "Jun 11, 2026",
        entries: &[
            "Smarter card suggestions: when you add cards, the picks that fit your deck's leader best now rise to the top automatically, no setup, it just orders the stack for you.",
            "Open the add-cards screen and it fills instantly with leader-matched suggestions, you no longer have to search first to start swiping.",
            "Suggestions are ranked by how well each card synergizes with your leader, so the most relevant cards come first and the long tail follows.",
            "Cards already in your deck (main, maybeboard, or sideboard) are hidden from the suggestions, so you only ever see what you can still add.",
            "Prefer a different order? Pick any sort (price, mana value, name, and more) and your choice always wins over the smart default.",
            "Import a full deck straight from a deck-list URL right onto your existing deck.",
            "New guided hints: your first visit to key screens shows a quick one-time tip (how to swipe, where to add or import cards), and a small ? button brings any tip back whenever you want.",
            "Swipe tips are color-coded by direction, so add, skip, maybeboard, and undo are obvious at a glance.",
            "An empty deck now offers one-tap Add cards and Import buttons right inside its warning.",
            "Resend verification email shows a live cooldown after sending, plus a Check again button that updates your verified badge on the spot.",
            "Security upgrade: changing your email, username, or password now sends you a notification email.",
        ],
    },
    Release {
        version: "1.0.2",
        date: "Jun 8, 2026",
        entries: &[
            "Faster searches across the catalog: results land in a fraction of the time on cellular.",
            "In-deck filters now narrow correctly: type, set, format legality, leader eligibility, and rarity sort.",
            "Card images render with cleanly rounded corners across every screen.",
            "Login, sign-up, and edit screens show loading states so taps feel acknowledged.",
            "Show/hide button on every password field.",
            "Loading skeletons replace blank flashes on deck lists, profiles, stats, and home flavor.",
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

/// The `major.minor` of a version string, e.g. "1.3.1" -> "1.3", "1.0.10" ->
/// "1.0". Slices the input, so a `&'static str` in yields `&'static str` out.
fn major_minor(version: &str) -> &str {
    version.rsplit_once('.').map_or(version, |(head, _)| head)
}

/// The release history: a major.minor chip filter over a newest-first list of
/// versions, each with its date and notes. Defaults to the latest release's
/// line; "All" shows everything. Renders just the content block (chips + list),
/// so wrap it in your own page or sheet chrome.
#[component]
pub fn Changelog() -> Element {
    // major.minor keys in display order (newest first), deduped, for the filter.
    let mut minors: Vec<&'static str> = Vec::new();
    for release in UPCOMING.iter().chain(RELEASES.iter()) {
        let key = major_minor(release.version);
        if !minors.contains(&key) {
            minors.push(key);
        }
    }

    // None = "All"; Some(key) narrows to one line. Defaults to the latest
    // released line (not the upcoming teaser); "All" stays an option.
    let mut selected = use_signal(|| RELEASES.first().map(|r| major_minor(r.version)));
    // Included in each card's key so switching filters remounts the visible
    // cards, replaying their ease-in animation.
    let filter_key = selected().unwrap_or("all");
    // Upcoming entries render first with an "Upcoming" badge; the first released
    // entry after them is the "Latest".
    let upcoming_count = UPCOMING.len();

    rsx! {
        div { class: "changelog-filter",
            button {
                class: if selected().is_none() { "chip selected" } else { "chip" },
                onclick: move |_| selected.set(None),
                "All"
            }
            for key in minors {
                button {
                    class: if selected() == Some(key) { "chip selected" } else { "chip" },
                    onclick: move |_| selected.set(Some(key)),
                    "{key}"
                }
            }
        }
        div { class: "changelog-list",
            for (i, release) in UPCOMING.iter().chain(RELEASES.iter()).enumerate() {
                if selected().is_none_or(|key| key == major_minor(release.version)) {
                    div { key: "{filter_key}-{release.version}", class: "changelog-card",
                        div { class: "changelog-version-row",
                            h2 { class: "changelog-version", "{release.version}" }
                            span { class: "changelog-date", "{release.date}" }
                            if i < upcoming_count {
                                span { class: "status-tag status-doing", "Upcoming" }
                            } else if i == upcoming_count {
                                span { class: "status-tag status-done", "Latest" }
                            }
                        }
                        ul { class: "changelog-bullets",
                            for entry in release.entries {
                                li { "{entry}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
