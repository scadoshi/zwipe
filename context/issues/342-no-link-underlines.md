# 342. No underlines on links

State: fixing | Owner: thinker | PR: - | Build seen: release with #526

## Report
Scotty, 2026-11-11, via the composer: a new link standard. No underlines anywhere: a link is the accent color with a subtle hover, as the portfolio and zwipe.net looked before 338. This reverses 338's underline (#526 merged in release, not deployed; portfolio#43 live).

## Fix
thinker, 2026-11-11.
- **orangui#33** (merged, 21a47fb): an unclassed link gets a soft default of the accent with `text-decoration: none`. Hover mixes the accent ink 30% toward the text color, a slight brighten on a dark theme and a slight deepening on a light one. Focus takes that color and a 2px accent ring. The banner and release-card links lost their underlines. Gate `tests/no_underlines.rs`.
- **zwipe** (this PR): repins orangui to 21a47fb. The footer note's links, the privacy and About-credit links, app.zwipe.net's tagline and zite's global `a` rule set the accent (or keep their ink) with no underline. zite gives unclassed links the accent, since its element rule outranks the kit's zero-specificity default. Classed text links (`.privacy-link`) take the same brighten and ring. The web's sheet copies are regenerated. Gate `components/tests/no_underlines.rs` covers every sheet the app, app.zwipe.net and zwipe.net load.
- **Portfolio** (its own PR): repins orangui and drops #43's resting underline and the footer link's underline.
- **Swept** with headless Chrome (hover-capable pointer, `:hover` and `:focus-visible` forced on every visible link): zwipe.net `/`, `/about`, `/guides`, a guide, `/changelog`, `/privacy`; app.zwipe.net `/login`, `/about`, `/privacy`; the portfolio's `/`, two project pages and Cairn's privacy page. 0 underlined states.
