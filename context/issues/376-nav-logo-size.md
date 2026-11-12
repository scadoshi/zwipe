# 376. The nav logo is a different size on each site

State: fixing | Owner: thinker | PR: - | Build seen: production, web (Scotty's recording switching between app.zwipe.net and zwipe.net)

## Report
Scotty, 2026-11-12: the nav's Z logo is a different size on app.zwipe.net than on zwipe.net. Then: the target is the portfolio's S (scottyfermo.com), and all three should come from one orangui nav-logo rule.

## Cause
Each site drew its mark with its own rule. All three are seven lines at a 0.9 line height, so the font size alone set the height:

| Site | Rule | Font | Rendered (w × h) |
|---|---|---|---|
| scottyfermo.com, S | portfolio `.nav-logo` | 0.4rem = 6.4px | 44 × 40.4 |
| zwipe.net, Z | zite `.nav-logo` | 0.35rem = 5.6px | 52 × 35.2 |
| app.zwipe.net, Z | zwiper-web `.web-nav-logo` | 1.85rem / 6.3 = 4.7px | 42 × 29.5 |

## Fix
thinker: orangui#37 adds one soft `.nav-logo` rule at 0.4rem in the logo color. zwipe bumps the pin, and zite and zwiper-web drop their own rules, so both Zs read the shared one; the portfolio does the same in its own PR. After, at 1280 and 390: S 44 × 40.4, both Zs 56 × 40.4 (14 columns to the S's 11), every nav 80px tall at 1280 (zwipe.net's was 75). The web app shows no logo in its phone nav, as before.
