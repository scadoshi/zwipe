# Store submission copy

One release's "What's New" text, in one place, ready to paste into both stores. Each version gets a directory with a `whats_new.md` holding the copy in a fenced block, so it can be copied without picking it out of a larger document.

## The 500-character rule

Play caps "What's New" at 500 characters. The App Store allows 4,000. Write to Play's limit and both stores take the same text, which is the whole point of this directory. Every file states its own character count.

## One text, both stores

Decided 2026-09-22: every release from 1.10.2 onward ships the same "What's New" to the App Store and Play. Write it once, paste it twice.

History varies, and that is fine. Of the 27 releases that went to both stores, only 5 used identical text. Those files keep both variants as submitted, because they record what is live rather than what we would write today. Nothing there needs fixing.

## Naming cards and features

Use the card game's own words. "Magic: The Gathering", "Commander", "Oathbreaker", "command zone", "Universes Beyond", "Secret Lair": these name things the user is looking at, and working around them produces copy that describes nothing. This covers store listings, release notes and any feature description, not just the notes in this directory. Casing follows [`../../development/ui_text_conventions.md`](../../development/ui_text_conventions.md), which has always spelled these out properly.

Both apps are approved and have cleared review many times with this vocabulary in the listing: the iOS subtitle is "Swipe to Build MTG Decks", and the keyword field carries MTG, Magic the Gathering, commander, EDH and Scryfall.

The one thing that stays generic is the app name. Apple's Guideline 4.1(a) rejection was about "Zwipe MTG", which became "Zwipe TCG" and cleared. That was the name, not the vocabulary, and the Play package name is permanent anyway. Do not put "Magic" in the title.

Some older Play notes avoided these words on the theory that Apple required it. Apple did not, and those entries stay as submitted.

## Writing a new one

Start from the `UPCOMING` block in `zwipe-core/src/content/changelog/mod.rs`, which is the in-app changelog and already in the right voice. Compress to fit 500, name the actual controls, and skip anything a user cannot see. Server-side work does not belong here.

Then paste the same text into App Store Connect and Play Console. Field positions and the rest of the listing copy live in [`../ios/app-store/submission/form_fields.md`](../ios/app-store/submission/form_fields.md) and [`../android/play-store/submission/form_fields.md`](../android/play-store/submission/form_fields.md).

## Releases

| Version | Shape | Characters |
|---------|-------|-----------|
| [1.10.2](1.10.2/whats_new.md) | shared | 496 |
| [1.10.1](1.10.1/whats_new.md) | per store | 331 |
| [1.10.0](1.10.0/whats_new.md) | per store | 418 |
| [1.9.3](1.9.3/whats_new.md) | shared | 388 |
| [1.9.2](1.9.2/whats_new.md) | per store | 489 |
| [1.9.1](1.9.1/whats_new.md) | per store | 386 |
| [1.9.0](1.9.0/whats_new.md) | per store | 372 |
| [1.8.1](1.8.1/whats_new.md) | per store | 383 |
| [1.8.0](1.8.0/whats_new.md) | shared | 388 |
| [1.7.6](1.7.6/whats_new.md) | per store | 338 |
| [1.7.5](1.7.5/whats_new.md) | shared | 484 |
| [1.7.4](1.7.4/whats_new.md) | per store | 252 |
| [1.7.3](1.7.3/whats_new.md) | per store | 408 |
| [1.7.2](1.7.2/whats_new.md) | per store | 346 |
| [1.7.1](1.7.1/whats_new.md) | per store | 443 |
| [1.7.0](1.7.0/whats_new.md) | per store | 451 |
| [1.6.0](1.6.0/whats_new.md) | per store | 464 |
| [1.5.0](1.5.0/whats_new.md) | per store | 526 |
| [1.4.0](1.4.0/whats_new.md) | per store | 578 |
| [1.3.1](1.3.1/whats_new.md) | shared | 207 |
| [1.3.0](1.3.0/whats_new.md) | per store | 941 |
| [1.2.1](1.2.1/whats_new.md) | per store | 350 |
| [1.2.0](1.2.0/whats_new.md) | per store | 709 |
| [1.1.4](1.1.4/whats_new.md) | per store | 522 |
| [1.1.3](1.1.3/whats_new.md) | per store | 565 |
| [1.1.2](1.1.2/whats_new.md) | shared | 111 |
| [1.1.1](1.1.1/whats_new.md) | per store | 357 |
| [1.1.0](1.1.0/whats_new.md) | per store | 805 |
| [1.0.10](1.0.10/whats_new.md) | App Store only | 330 |
| [1.0.9](1.0.9/whats_new.md) | App Store only | 1022 |
| [1.0.7](1.0.7/whats_new.md) | App Store only | 944 |
| [1.0.6](1.0.6/whats_new.md) | App Store only | 1413 |
| [1.0.2](1.0.2/whats_new.md) | App Store only | 534 |
