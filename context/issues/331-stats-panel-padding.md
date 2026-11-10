# 331. Deck stats panels let their content touch the rules

State: fixing | Owner: scotland | PR: #511 | Build seen: live (Surfer, 1920x907), local release 917a66843 (scotland)

## Report
Scotty and Surfer, 2026-11-10, web deck page: in the Distributions, Mana and Draw odds panels the content sits on the panel's rules. Surfer measured, live at 1920x907: Distributions' "See more" (`.chart-see-more`) on the bottom border (0), Mana's last fulfillment row 3.6 from the bottom, Draw odds' stepper right under the header rule (0) and its "See more" on the bottom border (0). Config, Account, Playstyle, Activity and Commander maybeboard are fine. Scotty wants generous bottom padding. Shots: `target/scotty-shots/331-*.png`, Surfer's `~/Developer/surfer-shots/331-*.png` on halo.

## Repro
1. A deck with cards, on the web, at 1260 wide (the panels as folds) or 390 (the accordion).
2. Open Distributions, Mana and Draw odds. Scotland, local, from the header rule to the first content and from the last content to the panel's bottom edge: at 1260 Distributions 16 / 1, Mana 16 / 3.8, Draw odds 0 / 1; at 390 Distributions 25.6 / 13, Mana 25.6 / 15.8, Draw odds 9.6 / 13.

## Diagnosis
The chart blocks pad only their sides (`padding: 0 0.75rem`), and on a wide window the stat bodies sit straight in `.deck-fold-inner`, which adds no vertical room. A settings row that opens a panel brings its own 1rem, so Distributions and Mana look right at the top; a chart that opens one (Draw odds' stepper) or closes one (every panel) touches the rule or the edge. The phone accordion and the app's section pad 0.75rem top and bottom, so it shows less there. Zite's shared deck already wraps the same bodies in `.deck-stat-body` (`padding: 0.75rem 0`).

## Fix
scotland, #511: the web's `DistributionsBody`, `ManaBody` and `DrawOddsStepper`, and the app's three stat sections, wrap their blocks in `.deck-stat-body`. That shared rule (components `magic.css`) puts a chart that opens the body 1rem under the rule and leaves 1.25rem at its foot. A settings row that opens it keeps its own padding, and rows stack on their own borders. Inside the phone accordion (`.web-section-body`) and the app's section (`.collapsible-section-body`), whose own padding already gives 0.75rem, the foot is 0.5rem. Measured after: 16 / 21, 16 / 23.8, 16 / 21 at 1260 and 25.6 / 21, 25.6 / 23.8, 25.6 / 21 at 390. Zite's shared deck takes the same rule: its stat bodies go from 0.75rem to 1rem / 1.25rem. Not tested by a unit test: the result is layout, measured headless. Verify: the Repro at 1260 and 390 shows room above the first block and under the last in each panel.
