# Google Play Console Submission

Text to paste into Google Play Console fields for **Zwipe TCG**. Mirrors the
iOS [form_fields.md](../../ios/app-store-submission/form_fields.md); where the two
stores share copy, this keeps it consistent so the brand reads the same on both.

The public listing stays **generic ("TCG")** — no "Magic", "MTG", "Commander",
"EDH", "Planeswalker", or "Scryfall" in store-visible copy. That's the same
positioning that cleared Apple's copycat scrub (Guideline 4.1(a)); Google Play
enforces equivalent IP/copycat rules, so we don't reintroduce those terms here.

---

## Create app (App details)

| Field | Value |
|-------|-------|
| App name | Zwipe TCG |
| Default language | English (United States) – en-US |
| App or game | App |
| Free or paid | Free |
| Package name (permanent — never changes) | `com.scadoshi.zwipe` (matches the Apple bundle ID) |

Plus the two declaration checkboxes: Developer Program Policies, and US export laws.

---

## Main store listing

### App name (max 30)

Zwipe TCG

### Short description (max 80)

Swipe right to add, left to skip — build trading-card decks with your thumb.

### Full description (max 4000)

Swipe through cards. Build decks fast. Zwipe turns the slow, cluttered desktop deck-building experience into something that fits in one thumb.

Swipe right to add a card, left to skip, up to stage it for later. Open a card's image with a tap. Filter by anything that matters — color identity, mana cost, type, oracle text, keywords, artist, set, rarity, mechanical role. The card pool updates as you swipe.

Built for the 100-card singleton format and the legendary creature that anchors it. Full support for alternate leader rules: partner leaders, special backgrounds, planeswalker-led signature spells, color identity validation, and a maybeboard for cards you haven't decided on yet.

Other tools:
- Multiple decks with card counts, mana curve stats, and price estimates
- Sideboard support
- Import and export decklists in standard text format
- 14 color themes with dark and light modes, including 3 colorblind-accessible options
- 110,000+ cards synced nightly
- Free to use, no ads, no microtransactions
- Your decks sync across sessions and devices

Built for players who want a fast, focused mobile experience — not another cluttered desktop tool squeezed onto a small screen.

### Graphics

| Asset | Spec | Status |
|-------|------|--------|
| App icon | 512×512, 32-bit PNG, ≤1 MB, no alpha | Use the 1.6× ASCII Z master (`zwiper/assets/favicon/icon-1024.png`), downscale to 512 |
| Feature graphic | 1024×500 PNG/JPG | TODO |
| Phone screenshots | 2–8, 16:9 or 9:16, each side 320–3840 px | Reuse iOS captures (re-export at a Play-accepted size) |
| 7"/10" tablet screenshots | optional | TODO (skip if phone-only) |
| Promo video | optional YouTube URL | TODO (the swipe demo) |

---

## Store settings

| Field | Value |
|-------|-------|
| App category | Entertainment (we selected "App", not "Game"; if switched to Game, use Card / Strategy) |
| Tags | deck builder, trading card game, cards |
| Email address (public) | TODO — confirm support address shown on the listing |
| Phone / Website | Website: https://zwipe.net (phone optional) |

---

## App content (Policy)

### Privacy policy URL

TODO — confirm/publish (e.g. https://zwipe.net/privacy). Required before release.

### App access  ⟵ "any other information required to access your app"

Zwipe needs an account to use, so reviewers must sign in. In Play Console choose
**"All or some functionality is restricted"**, then add one access instruction:

| Field | Value |
|-------|-------|
| Name | Reviewer login |
| Username | applereview |
| Password | (do not commit — enter manually in Play Console) |

**Any other information** (paste into the free-text box):

> Zwipe requires a single account to use; there is no separate guest mode, so please sign in with the credentials above to reach full functionality.
>
> To sign in:
> 1. Launch Zwipe.
> 2. On the welcome screen, tap "Log in" (not "Sign up").
> 3. Enter the username and password listed above.
> 4. Tap "Log in" — this opens straight into the app.
>
> The account uses a standard username + password login only. To be explicit about the restricted-access examples:
> - No two-step verification and no one-time codes.
> - No QR codes or barcodes.
> - No biometric login (no fingerprint or face recognition).
> - No location-based access, memberships, or paywalls.
>
> An internet connection is required so the app can search and browse the card catalog.

### Ads

No — the app does not contain ads.

### Content rating (IARC questionnaire)

Category: Reference, News, or Educational? No — submit as the matching app/game
type. Mirror the iOS age-rating answers: infrequent fantasy violence, infrequent
mild horror/suggestive themes, weapons references in card art; no real-money
gambling, no user-to-user communication. Email for the questionnaire: developer
contact above.

### Target audience and content

Target age: 13+ (not directed at children). No appeal to children.

### Data safety

TODO — complete the Data safety form. Starter facts to declare:
- Account login collects: email address, username (for account function).
- Decks/preferences stored on our server (synced across devices).
- Data encrypted in transit (HTTPS).
- Users can request account + data deletion in-app (Profile → More → Delete account).
- No data sold; no advertising/third-party sharing.

### Other declarations

| Question | Answer |
|----------|--------|
| Government app | No |
| Financial features | No |
| Health | No |
| News app | No |

---

## Release

| Field | Value |
|-------|-------|
| Track | Internal testing first → Closed/Production |
| Version (client) | 1.2.1 (`CARGO_PKG_VERSION`, aligns with the min-version gate) |
| Signing | Play App Signing (let Google manage the app signing key; upload key generated at build) |
| Copyright | 2026 Scotty Fermo |

### Release notes (What's new — max 500 chars, store-visible)

Keep generic per the copycat note at the top (no "Commander", "Planeswalker", etc.).

**1.2.1:**

- Tap the new eye button while swiping to read a card's full rules and stats: rules text with real symbols, type, rarity, keywords, and power/toughness or loyalty. Great for alternate-art or textless printings that hide the details.
- Smoother startup: the app opens straight into its themed layout, with no flash of unstyled content.

**1.2.0:**

- Draw-odds: see your chance of drawing a land, ramp, or removal in your opening hand or by any turn.
- Set a power level for your deck, and add descriptive tags like Budget, Jank, Meme, or Precon.
- Many more strategy tags, each with a plain-language definition.
- Toggle synergy suggestions on or off on the add-cards screen as you build.
- Contradictory include/exclude filters are now caught before you apply them, and lands drop out of the add screen once you hit your land target.

**1.1.4:**

- Set a land target for your deck and get a heads-up while building if you drop below it.
- Build on a budget: filter cards by price (USD, EUR, or Tix), and set a price target to get alerts as your total nears and passes it.
- Your deck's Stats, Distributions, Mana, and Warnings are now grouped into tap-to-expand sections.
- Cleaner, more consistent filter controls, plus a fix for a flicker when the app first opens.

**1.1.3:**

- Card names now show while you swipe, so every card is identifiable at a glance — including alternate and non-English printings.
- A smoother deck builder: tap a field to choose, empty fields read "Not set," and your deck name is checked as you type.
- More strategy tags with quick definitions, plus power-level options.
- Read the Privacy Policy right inside the app, from your Profile.
- Clearer sign-in and profile forms, with errors shown under each field.

**1.1.2:**

- A small consistency fix to the card-swipe screens so the filter controls look and behave the same everywhere.

**1.1.1:**

- Get help without leaving the app: every screen now has a Help button — tap it to report a problem (opens your email) or join our Discord community.
- New quick tips on the import and export screens guide you through importing or sharing a decklist.
- Bug fixes and reliability improvements.

**1.1.0** (mirrors the iOS 1.1.0 What's New, reworded for the generic listing):

- Tag your decks: add up to 5 strategy tags to label a deck at a glance — shown on your deck list and deck page.
- Swipe to set your deck's leader, partner, background, or signature spell instead of searching.
- Tap a card in your deck to expand its full details: cost, type, rules text with real symbols, and stats.
- Keyword helper: tap a keyword for a quick, plain-language reminder of what it does.
- Polish across card art, symbols, and dialogs.

---

## Notes

- Package name `com.scadoshi.zwipe` is **permanent** on Play — it can never be
  changed after the first publish. It deliberately matches the Apple bundle ID.
- Public listing copy stays generic per the copycat note at the top of this file.
- The server enforces a minimum client version (`MIN_CLIENT_VERSION`, live at
  `0.0.0` = open); ship the Play build at 1.1.0 or later so it isn't gated.
