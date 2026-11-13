# 435. The Stacks float pops up and sways like a metronome, and a tap reveal snaps open

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: 5f4ceccf7 | Type: issue

## Report
Scotty on the phone app's Stacks float (421): the cards moving aside are fine, then the card pops up with little ease, and the sway tilts back and forth like a metronome. It should feel like it is floating: start sooner, blended in while the cards move aside, ease the lift in softly, smoother turns, a gentle bob on a different period from the tilt so it never repeats on a beat, a smaller turn; keep the ease back, nothing under reduced motion, transform only, and give web hover the same feel. droider on 428 (narrow web touch Stacks): the reveal snaps open in one frame with the lift after, next to a quick settle; the reveal should ease open too.

## Cause
The float's lift and growth ran on their own 0.3s transitions after the 500ms reveal (`--float-delay` was the whole move), while only the turn's reach eased, so the card jumped up once the cards had moved. The sway was one tilt on one period. In tap mode a reveal drops the tuck margin of the card after the revealed one, a layout change with no transition, so the column opened in one frame and only the page's 250ms settle slid what followed.

## Fix
laptop, in `magic.css`. One eased number, `--float-reach`, now carries the whole float (lift, growth, turn and bob) in and out over 0.8s on a soft ease-in-out, so nothing pops; the turn (`float-sway`, 6.2s) and a new bob (`float-bob`, 3.9s, `--float-bob-lift`) run on periods that never line up; the default turn drops to 0.8 degrees, the stack's to 0.4 with a 0.08rem bob; the stack's float starts a quarter of the way into the move (`--float-delay: calc(var(--card-stack-move) / 4)`). In tap mode the slots also transition `margin-top` on the stack's own move and curve, so a reveal and its settle ease open and closed, carrying the column below.

## Checked (headless Chrome, app.zwipe.net local, 390 touch)
- Sampled per frame: the next slot's margin eases from -211px to 0 in the move's curve; the float starts at about 125ms and reaches full lift, growth and turn by about 950ms with no jump; a tap off eases it back.
- Before and after clips at 390 (`435-before.mp4`, `435-after.mp4`).
- The app draws its stacks with the same `CardStack` in tap mode and the same sheet.

## Verification
-
