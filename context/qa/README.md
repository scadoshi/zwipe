# QA log

Raw reports from the factory's QA agents, copied here after each pass so they survive the machines they ran on. Screenshots stay on those machines and aren't committed: Surfer's in `~/Developer/surfer-shots` on halo, Droid's in `~/Developer/droid/shots` on the laptop.

- [`surfer-report.md`](surfer-report.md): Surfer, the three websites (app.zwipe.net, zwipe.net, scottyfermo.com) in Chrome on halo. Parts 1 to 3 are wide, 4 is 720px, 5 is 504px, and 6 onward are fix checks after a deploy.
- [`droid-report.md`](droid-report.md): Droid, the Android app on a Pixel 9a emulator against production. Pass 1 is a full sweep, and pass 2 onward are rechecks.

Each finding was triaged with Scotty. What became work is in [`../progress/web_nits.md`](../progress/web_nits.md), where items carry Surfer's numbers as [S<n>] and Droid's as D<n>.

## Passes

| When | Agent | Pass | Against | Result |
| --- | --- | --- | --- | --- |
| 2026-11-06 | Surfer | Parts 1 to 3 | main `b0d713965` | 31 issues; triaged into 105 to 127 |
| 2026-11-06 | Surfer | Part 4, 720px | main `25ff79ae0` | 9 new; into 132 to 138 |
| 2026-11-06 | Surfer | Part 5, 504px | main `25ff79ae0` | 5 new; into 140 to 144 |
| 2026-11-06 | Droid | Pass 1 | release | 38 issues; into 145 to 147, plus extensions |
| 2026-11-06 | Surfer | Part 6, fix check | main `811c3bbbe` | Most landed; 125 and 118-narrow open; 52 to 54 new |
| 2026-11-06 | Droid | Pass 2, recheck | main `60d705459` | running |
| 2026-11-07 | Surfer | Part 7, fix check, 1450 only | main `60d705459` | 122, 133 (wide), 144 (reload), 147 (hero), 150, 152 fixed. Blocked by #55: a second app tab logged both tabs out. #56: the login page counters differ from Home and zwipe.net. |
