# 281. Leave guard blocks leaving with no dialog

State: closed (can't reproduce) | Owner: - | PR: - | Build seen: 1e03520ef

## Report
Scotty, web: on Create deck with unsaved edits, Back and every other way out do nothing and no "leave this form?" dialog shows. Leaving only works after clearing the edits. Suspects: #409 (Back closes overlays first) or the guard from 210 (#346). Surfer is reproducing on live.

## Repro
Pending Surfer.
Surfer, live web 1e03520ef (wide, hard reload): can't reproduce. With unsaved edits every exit (header Back, logo, nav links, browser Back) shows "Leave without saving?" with Stay and Leave; with no edits every exit leaves at once. Edit deck matches. Two quirks: with a Format/Commander side sheet open, the first click only closes the sheet; after browser Back is held, the tab title changes to "Decks | Zwipe" while the URL stays /decks/new. Waiting on Scotty: which exit, which field, phone app or web, which width.

## Verification
Scotty's screenshot on live 1e03520ef shows "Leave without saving?" with Stay and Leave on Create deck. Scotty: it took a hard refresh; the browser was running the old bundle (see 282). Closed, can't reproduce.
