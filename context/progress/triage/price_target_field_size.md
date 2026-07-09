# Make the price-target field smaller (it's a float)

**Source:** self-noted, 2026-06-29.
**Verdict:** decided — small UI tweak.

The price-target (budget) input is sized like a general text field, but it only
ever holds a price float (e.g. `150` / `150.00`). Shrink the field to fit the
expected value — a money amount doesn't need a full-width input.

Field lives in the deck form: `zwiper/.../screens/deck/components/deck_fields.rs`
(display in `deck_profile.rs`). Same likely applies to the land-target stepper if
it reads oversized; check while in there.
