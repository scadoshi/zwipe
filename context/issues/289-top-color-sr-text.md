# 289. Playstyle's Top color pip reads Mono-red to screen readers

State: verified (easy) | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Surfer, live web: Playstyle Summary's Top color row (decks containing red) has a pip whose sr-only text is 'Mono-red'; it should read 'Red'. Top identity's 'Mono-red' is right.

## Fix
Branch `nits/laptop-289`. Web's `IdentityPips` gives a single color no visible name but a screen-reader name, "Mono-red", which is right for an identity (the Show chips). The Top color row already shows the color's name ("Red") before its pip, so screen readers heard "Red Mono-red". `IdentityPips` takes `speak` (on by default), and the Summary row turns it off whenever it shows a name, so the row reads "Red, 4 decks". The app's pips carry no screen-reader name for a single color, so its row already read "Red". Failing test first: `a_single_color_beside_its_shown_name_speaks_nothing_more`.
