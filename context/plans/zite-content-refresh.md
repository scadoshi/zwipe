# Zite Content Refresh

Update zwipe.net page content to reflect the current state of the app. Several pages have outdated copy from before major features shipped.

---

## Page-by-Page Audit

### home.rs — Landing Page

**Current features grid (6 cards):**

| # | Current | Stale? | Update |
|---|---------|--------|--------|
| 1 | "Swipe to build" — right to add, left to skip | Partially | Add up-swipe for maybeboard |
| 2 | "Deep filters" — color, type, mana cost, oracle text, artist, set | Partially | Add commander eligibility, per-section clear, keywords, produced mana |
| 3 | "35k+ cards" — full scryfall, synced nightly | OK | No change needed (will become 80k+ after multi-printing sync) |
| 4 | "Commander ready" — singleton mode, commander assignment, mana curve, color identity | **Stale** | Now supports partner, background, signature spell, oathbreaker, 4 partner variants, commander eligibility filtering, warning actions |
| 5 | "Import / export" — paste from moxfield or archidekt | Partially | Now supports maybeboard sections (`// Maybeboard` header) |
| 6 | "Your decks, synced" — account-based, across sessions | OK | No change |

**Proposed updated features grid:**

1. **Swipe to build**
   - "browse cards one at a time. right to add, left to skip, up for maybeboard."

2. **Deep filters**
   - "filter by color, type, mana cost, oracle text, keywords, artist, set, rarity, commander eligibility, and more. per-section clear buttons for fine-tuned control."

3. **35k+ cards**
   - No change (or update to reflect default_cards count once multi-printing ships)

4. **Commander ready**
   - "full commander support — partners, backgrounds, oathbreaker with signature spell. commander eligibility filtering per format. color identity validation across the command zone."

5. **Import / export**
   - "paste any decklist from moxfield or archidekt. maybeboard sections import and export automatically."

6. **Your decks, synced**
   - No change

**Also consider adding a 7th or 8th card:**

7. **Maybeboard**
   - "stage cards you're considering without committing them to the deck. swipe up to maybe, move to deck when ready."

8. **9 themes**
   - "dark mode by default. choose from 9 color themes to match your style."

---

### about.rs — About Page

**Stale sections:**

| Section | Current | Update |
|---------|---------|--------|
| Architecture | "zerver crate doubles as shared type library — zwiper depends on it with server features disabled" | **Wrong** — zwipe-core is now the shared crate. Zerver and zwiper both depend on zwipe-core. The feature-flag approach was replaced. |
| Testing | "250+ unit tests" | **Stale** — now 308+ tests, ~100 in zwipe-core |
| Frontend — zwiper | No mention of maybeboard, partner/background, command zone | Add key features |
| Backend — zerver | No mention of maybeboard, partner fields, WarningAction | Add key features |
| Card Data | "35k+ cards" | Will change with multi-printing |

**Proposed updates:**

**Architecture section — replace with:**
"hexagonal (ports & adapters) throughout. domain logic lives in zwipe-core — a pure shared crate with no framework dependencies. zerver and zwiper both depend on zwipe-core for shared types. inbound adapters (http handlers, ui screens) and outbound adapters (sqlx repositories, api clients) are swappable."

**Testing section — replace with:**
"308+ unit tests across domain logic, value object validation, deck validation, and import parsing. ~100 tests in zwipe-core covering commander eligibility, partner validation, deck metrics, and card filtering. newtypes enforce correctness at construction — `UserId`, `DeckId`, `EmailAddress`, `Password` — so invalid states can't be passed around."

**Frontend — zwiper — add to description:**
"maybeboard support (swipe up to stage, move between active and maybe). partner, background, and signature spell fields with conditional visibility. command zone display with toggle. tri-state maybeboard filter on remove screen. per-section filter clear buttons."

**Backend — zerver — add to description:**
"partner_commander_id, background_id, signature_spell_id on deck profiles. maybeboard boolean on deck cards with metrics/validation exclusion. warning action system (fix quantity, clear commander). commander eligibility validation per format."

---

### ios.rs — iOS Download Page

**Current:** "app store — pending" / "submitted and under review"

**Update when approved:** Replace with redirect to App Store link. Until then, current copy is fine but could mention specific features:

"zwipe has been submitted to the app store. while it's under review, here's what's waiting for you: swipe-based deck building, commander support with partners and backgrounds, maybeboard, 9 themes, and 35k+ cards synced nightly."

---

### android.rs — Android Download Page

**Current:** "google play — pending" / "submitted and under review"

**Same approach as iOS.** Update copy to mention features. When approved, redirect to Play Store link.

---

### contribute.rs — Contribute Page

**Current copy is fine.** No stale content. The three contribution options (Stripe, Buy Me a Coffee, GitHub Sponsors) are still accurate.

**Optional enhancement:** Add a brief "what your support funds" section:
- Server hosting (Ubuntu, PostgreSQL, Cloudflare)
- Nightly Scryfall sync infrastructure
- Ongoing development of new features

---

### discord.rs — Discord Page

**Current copy is fine.** Link is active, description is accurate.

---

### privacy.rs — Privacy Policy

**Current copy is mostly fine.** One potential update:

- "march 2026" → update to current month when content ships
- Consider adding mention of maybeboard data (stored alongside deck cards, same retention policy)
- No new third-party services added, so that section is still accurate

---

### verify.rs + reset.rs — Auth Pages

**No changes needed.** These are functional pages with status messages, not marketing content.

---

### main.rs — Nav Bar

**Current links:** about, contribute, discord, app store, play store

**Consider adding:** A "features" section link that anchors to the features grid on the home page, or a dedicated features page if the grid gets long enough.

---

## Priority Order

1. **home.rs** — Most visible, most stale (commander features, maybeboard missing)
2. **about.rs** — Architecture section is factually wrong (still references old feature-flag pattern)
3. **ios.rs / android.rs** — Low priority until store approvals come through
4. **Other pages** — Fine as-is

---

## Files Modified

| File | Change |
|------|--------|
| `zite/src/pages/home.rs` | Update 3 feature cards, optionally add 1-2 new ones |
| `zite/src/pages/about.rs` | Fix architecture section, update test count, add new feature mentions |
| `zite/src/pages/ios.rs` | Optional: add feature preview text |
| `zite/src/pages/android.rs` | Optional: add feature preview text |
| `zite/src/pages/privacy.rs` | Update date header |
