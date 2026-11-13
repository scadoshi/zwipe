# 492. Switching a deck's format away and back loses its commander

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: release 950ff3333

## Report
Scotty, via the composer: in deck edit and create, switching Format to one that lacks a slot (Commander to Oathbreaker) clears it. Switching back within the same edit restores what was cleared (the commander, partner and so on). Only within that open form; nothing is kept beyond it. Web and app.

## Before
- **surfer (web, 1440, Surfer 399 Isshin):** Edit, Commander to Oathbreaker to Commander, each applied, leaves the Commander slot 'Not set', 3 of 3. Commander to Standard to Commander, the same, 3 of 3. New deck the same, 3 of 3.
- **droider (app, Pixel):** Isshin gone after Commander to Oathbreaker to Commander, 3 of 3.
- **Mine (local web, Deck 1 with Atraxa):** 3 of 3.

## Cause
A format pick clears the commander (and with it the partner and background) and, unless the new format has one, the signature spell, and nothing remembers what was cleared:
- the app: `zwipe_ui::deck::form::FormValues::pick_format`, driven by `screens::deck_form::State::set_format`;
- the web: its own `DeckForm::select_format` and `clear_format` in `zwiper-web/src/lib/inbound/screens/deck/form/mod.rs`.

## Fix
laptop:
- **`zwipe_ui::deck::form::LeftZones<Z>`:** keeps, for an open form, each format's command zone as the form last left it, and hands it back when that format is picked again.
- **The app's model:** `State::set_format` keeps the zone it leaves (its `ZoneSnapshot`) and restores the new format's.
- **The web form:** `DeckForm` keeps each slot's card and field text and does the same.
- **Scope:** nothing is kept beyond the form. A new form starts with none, and a save or a leave drops it.
- **Tests:** `a_format_picked_back_within_the_form_brings_its_command_zone_back` (red before), `a_new_form_brings_back_nothing_from_another`, and `LeftZones`' own test.

## Checked
Local web, Deck 1 (Commander, Atraxa), edit, Commander to Oathbreaker to Commander, each applied, 3 of 3: Atraxa is back in the Commander field.

## Verification
-
