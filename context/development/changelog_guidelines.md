# Changelog Guidelines

The changelog lives in `zwipe-core/src/content/changelog/mod.rs` and is rendered on zwipe.net, in the app, and by the App Store notes. People scan it. Every entry is a phrase that says what changed, and then it stops.

## The rules

- **No period at the end.** No exclamation mark, no ellipsis either. An entry is a phrase, not a sentence. `no_entry_ends_with_a_period` fails the build otherwise.
- **No examples.** No "like Sol Ring", no "e.g.", no "such as", no list of the cards it affected. If the change needs an example to make sense, the guide is where that goes. `no_entry_gives_an_example` catches the common phrasings.
- **Say what changed and move on.** One clause is the target. A second clause is allowed only when the entry means nothing without it: "Clearing a filter works; Apply used to refuse an empty filter". Never a second sentence that explains the reasoning, the rule behind it, or how it used to work in detail.
- **No em dashes.** Use a semicolon, a comma, or a colon.
- **One entry per change.** Do not merge two changes into one entry with "and", and do not split one change across two entries.
- **Proper nouns keep their case.** Screen and section names as they appear in the app (Profile, Deck tags, Oracle tags). Formats in Title Case. Otherwise sentence case, per `ui_text_conventions.md`.
- **Shipped entries are history.** Reword them for these rules when you touch the file, but never change what they claim.

## Good

- `Messages stay up longer`
- `Shaking the phone no longer opens the system Undo prompt`
- `Android: switching between light and dark mode no longer closes the app`

## Not good

- `Messages stay up longer, so there is time to read one or open the stack before it goes.` (period, and a reason)
- `Commander select no longer offers melded cards like Brisela or Titania, Gaea Incarnate.` (examples)
- `Oathbreaker signature spell select no longer offers Adventure creatures. The card is a creature, only its Adventure half is a spell, and the format's rules say it can't be your signature spell.` (a second sentence explaining the rule)

## Mechanics

New work goes under `UPCOMING`. At cut time the entry moves to the top of `RELEASES` with the App Store's "Ready for Distribution" date, and the version bumps per `versioning.md`.
