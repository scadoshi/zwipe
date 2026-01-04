# Tasks

## Completed
- CSS color variable system (reduced contrast with solid hex colors)
- Stepper controls for power/toughness filters (replaced text inputs)
- Generic FilterMode enum (consolidated Exact/Within toggle pattern)
- Narrower CMC input fields with centering
- Option<i32> pattern for filters (None = no filter, Some = active)
- Clear button sets to None not zero
- Within mode partial values (defaults unset to 0)
- Reduced stepper sizes by ~20% to fit 85% accordion width

## In Progress / Bugs
- **Combat flickering bug**: Setting min/max in Within mode causes cards to flick continuously. Attempted fix (batch writes) didn't resolve. Need to investigate further - may be reactive loop elsewhere.
- **CMC filter bug**: Can't type values - gets cleared. Attempted fix (track previous state) didn't resolve. Alternative approach: only allow setting when BOTH min and max are set, leave as None otherwise.

## Pending Enhancements
- Update toast styles for overlay stacking
- Toasts should dissipate when changing screens

## Notes
- Accordion content width set to 85% (user preference)
- Using Dioxus 0.7 signals with use_effect for reactive syncing
- CardFilterBuilder uses builder pattern in domain layer
