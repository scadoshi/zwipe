# Google Play Console Submission

Text to paste into Google Play Console fields for **Zwipe TCG**. Mirrors the
iOS [form_fields.md](../../../ios/app-store/submission/form_fields.md); where the two
stores share copy, this keeps it consistent so the brand reads the same on both.

The **app name** stays generic: "Zwipe TCG", never "Zwipe MTG". That is what
Apple's Guideline 4.1(a) rejection was about, and the Play package name is
permanent regardless.

Descriptive copy is a different question, and since 2026-09-22 it uses the
app's own vocabulary. "Commander", "command zone", "Universes Beyond" and the
rest name things the user is looking at. Both apps are approved and the iOS
listing has cleared review many times carrying MTG, Magic the Gathering,
commander, EDH and Scryfall in its subtitle and keywords. Earlier notes here
claimed Apple required the scrub to extend that far; it did not.

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

Swipe right to add, left to skip. Build Commander decks with your thumb.

### Full description (max 4000)

Same text as the App Store description.

Swipe through cards. Build decks fast. Zwipe turns the slow, cluttered desktop deck-building experience into something that fits in one thumb.

Swipe right to add a card, left to skip, up to stage it for later. Open a card's image with a tap. Filter by anything that matters: color identity, mana cost, type, oracle text, keywords, artist, set, rarity, price, power and toughness, card roles, and thousands of community oracle tags. The card pool updates as you swipe.

Suggestions that fit your deck: open the add screen and the cards that suit your leader best are already on top, ranked by synergy and sharpened by what the community actually builds.

Know what every card does: tap any tag for a plain-language definition, browse the full tag dictionary, and swipe through real example cards that use it.

Built for Commander: 100-card singleton decks built around a legendary creature. Full support for partner commanders, Backgrounds, Oathbreaker signature spells, color identity validation, and a maybeboard for cards you haven't decided on yet.

Other tools:
- Deck stats at a glance: mana curve, type and color distributions, average power and toughness, and draw odds by turn
- Set a land target and a price budget, and get warned as you drift
- Star your deck's MVPs and share any deck with a public link
- Import and export decklists in standard text format, or import straight from a deck-list URL
- 31 color themes with dark and light modes, including colorblind-accessible options
- 110,000+ cards synced nightly
- Free to use, no ads, no microtransactions
- Your decks sync across sessions and devices

Built for players who want a fast, focused mobile experience, not another cluttered desktop tool squeezed onto a small screen.

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

TODO: confirm/publish (e.g. https://zwipe.net/privacy). Required before release.

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
> 4. Tap "Log in"; this opens straight into the app.
>
> The account uses a standard username + password login only. To be explicit about the restricted-access examples:
> - No two-step verification and no one-time codes.
> - No QR codes or barcodes.
> - No biometric login (no fingerprint or face recognition).
> - No location-based access, memberships, or paywalls.
>
> An internet connection is required so the app can search and browse the card catalog.

### Ads

No, the app does not contain ads.

### Content rating (IARC questionnaire)

Category: Reference, News, or Educational? No; submit as the matching app/game
type. Mirror the iOS age-rating answers: infrequent fantasy violence, infrequent
mild horror/suggestive themes, weapons references in card art; no real-money
gambling, no user-to-user communication. Email for the questionnaire: developer
contact above.

### Target audience and content

Target age: 13+ (not directed at children). No appeal to children.

### Data safety

TODO: complete the Data safety form. Starter facts to declare:
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
| Version (client) | 1.10.1 (`CARGO_PKG_VERSION`, aligns with the min-version gate), versionCode 42 |
| Signing | Play App Signing (let Google manage the app signing key; upload key generated at build) |
| Copyright | 2026 Scotty Fermo |

### Release notes (What's new: max 500 chars, store-visible)

Lives in [`../../../store-submissions/`](../../../store-submissions/), one
directory per version, shared with the App Store. Paste from
`store-submissions/<version>/whats_new.md`.

That 500-character cap is the binding one, so it is what the shared text is
written to. From 1.10.2 on, Play and the App Store get the same text.

---

## Notes

- Package name `com.scadoshi.zwipe` is **permanent** on Play, it can never be
  changed after the first publish. It deliberately matches the Apple bundle ID.
- Public listing copy stays generic per the copycat note at the top of this file.
- The server enforces a minimum client version (`MIN_CLIENT_VERSION`, live at
  `0.0.0` = open); ship the Play build at 1.1.0 or later so it isn't gated.
