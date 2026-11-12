# 381. Sheets and drawers slide out empty, and the Filter accordion drops shut

State: fixing | Owner: laptop | PR: - | Build seen: audit build 9680dcacb

## Report
Ease audit, rows 4 and 7 (`~/ease-audit/README.md`), moved from thinker via the composer: the Filter sheet, the Stacks card drawer and the Format picker remove their contents within a frame of closing, then slide an empty panel out for 300ms. A Filter section's close drops 517px in one frame, though its open slides. The theme menu keeps its contents while it closes, which is the behavior to copy.

## Cause
`DrawerSheet`, which all three use, mounted its children only while open. The accordion mounted a section's rows only while that section was open, so its collapse had nothing left to shrink.

## Fix
laptop, web (components):
- `DrawerSheet` keeps its content mounted from opening until the 300ms slide out ends, unless the sheet opens again first. Like the theme sheet, it slides away full.
- The card filter keeps a section's rows drawn while that section eases shut, whether it was closed by its header, by opening another section, or by Apply.
- Off the browser (the app) nothing is timed, so content drops at once as before.

## Checked (headless Chrome, 1422)
A probe sampled every frame of each close, counting frames where the panel is still on screen with an empty content area, and the accordion's height. Release, then this branch:

| Close | Release | This branch |
|---|---|---|
| Filter sheet × | 18 of 18 visible frames empty | 0 of 18 |
| Stacks drawer × | slides out empty (audit) | 0 of 19 |
| Format picker × | 18 of 18 empty | 0 of 18 |
| Filter section close | 574px to 0 in 1 frame | eases over 17 frames |

Under reduced motion the sheets close at once, as before. The probe is `target/381-sheet-probe.js` on the laptop.

## Verification
-
