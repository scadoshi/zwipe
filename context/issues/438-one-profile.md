# 438. At phone width the shared deck's Profile isn't the owner's, and a shared deck hides its warnings and Export

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: 3eac55fb2 | Type: issue

## Report
Scotty, corrected: the reference is the phone app's deck Profile, which shows no cards, only the deck's info and its stat panels. At phone width the owner's web Profile keeps the app's structure (info rows and stat panels), and the shared deck's Profile changes to match it, read only: info rows (name, format, commander, who shares it) and the stat panels in the app's style, with no featured strip and no fact-chip head (the cards live in the Cards layer, 430). Both use the app's panel style. A shared deck shows its Warnings read only (no Fix) and the Export cards action, and the shared page has one h1. Wide web unchanged. The owner's and the shared page are one thing.

## Fix
laptop.
- zwipe-ui: `deck::profile` holds `DeckInfo` (the Profile card's rows, from the owner's profile or a shared deck, with who shares it) and `shared_warnings` (the format's rules over a shared deck, as the owner's deck raises them); `deck::export::shared_text` writes a shared deck's decklist through the same builder as the owner's.
- Web: `DeckProfilePhone` is the phone Profile for both (the Profile card, the accordion with Budget open, Warnings, Data, the owner's Danger zone) and `DeckFoot` the wide foot for both; the sections take `DeckInfo` and `StatsProfile` in place of the owner's `DeckProfile`, and Warnings take the owner's fixes as an `Option`, so a shared deck's read only. `DataPanel` takes its routes, Import only for the owner. A shared deck gets `/:owner/deck/:slug/:token/export`, the owner's Export screen over the shared decklist. The deck's head is the page's h1 wherever it shows (wide, and atop the Cards), and the screen's hidden title steps aside there.

### 430 follow-ups (droider, halo)
- A tab opened straight on `?view=cards` gets the Profile put under it (`use_cards_route` replaces the entry with the Profile and pushes the Cards), so Back lands on the Profile.
- The Cards layer slides in from the right and back out the same way, on the web and in the app (`use_layer_presence` in components: the layer stays mounted while it slides out and drops on its own `animationend`, so it needs no clock; `magic.css` holds the slide).
- The web layer sits under the nav, which takes its title, `?` (the Cards' hint) and back arrow through `use_overlay_nav`, beside the menu, as the app's header has them.
- One loaded deck: on the web the phone Profile reads the editor's live deck, as the wide page's facts do, so an edit in the layer shows on it at once. In the app the deck view holds its deck in `HeldDeck`; the Cards screen answers its own deck fetch from it (no fetch, no ghost) and the deck view loads again in place when the layer closes. `Zone::of_deck` (zwipe-ui) reads the command zone from a deck for both.

## Checked (headless Chrome, app.zwipe.net local)
- Shared deck at 390: the Profile card (Name, Shared by, Cards, Format, Commander, Power level), Budget, Tags, Distributions, Mana, Draw odds, Data with Export alone; no head chips or featured strip; one h1. "Krenko goblins" shared shows the owner's same six warnings with no Fix.
- Shared Export opens at `/<owner>/deck/<slug>/<token>/export` with the decklist.
- Owner at 390: the same Profile; an edit in the Cards layer (+1) reads 48 on the Profile at once, and Undo puts back 47.
- A fresh tab on `?view=cards` (history of one): Back lands on the Profile.
- The layer slides in and out; the nav over it shows Back, `?` and the menu.
- Wide 1450: the shared foot shows Warnings read only and Data; one h1 on both pages.

## Verification
-
