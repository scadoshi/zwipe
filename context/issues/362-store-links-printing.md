# 362. Store links send names only, not the printing picked

State: research | Owner: thinker | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-11: should the store links carry each card's chosen printing, so people buy the version they picked? Research only: what TCGplayer's mass entry and Card Kingdom's builder accept, with a real test link each, and whether deck cards keep the chosen printing. No code until confirmed.

## Findings
Our data: a deck card stores the exact printing (`DeckCard.scryfall_data_id`), and its card carries Scryfall's `set` code, `collector_number` and `tcgplayer_id`. Foil is not recorded: a deck card has no finish, and import strips `*F*`.

TCGplayer mass entry (`/massentry?c=`, lines joined by `||`), from the page's own script (`MassEntry-*.js`), three line forms in this order:
- `{qty} {name} [{SET}] {number}`: `1 Delver of Secrets [INR] 60`
- `{qty} {name} [{SET}]`: `1 Sol Ring [LEA]`
- `{qty}-{productId}`: `1-609611`, the exact TCGplayer product
- No per-line foil: foil and normal are one cart-wide "Printings" checkbox, not in the URL. Set codes are TCGplayer's own and can differ from Scryfall's, and some old products carry no number (Alpha Sol Ring, product 1263).
- Syntax verified from the page's code; whether the filled cart keeps to the set or product was not observed.
- Test links: `https://www.tcgplayer.com/massentry?productline=Magic&c=1-1263||1-609611` (product ids) and `https://www.tcgplayer.com/massentry?productline=Magic&c=1%20Sol%20Ring%20%5BLEA%5D||1%20Delver%20of%20Secrets%20%5BINR%5D%2060` (set and number).

Card Kingdom builder (`/builder?c=`): name only. Its help lists `4 Ancient Den`, `4x Ancient Den` and `Ancient Den`; every set or foil marker tried against the builder's `/api/builder` (`[LEA]`, `(Alpha)`, `(INR) 60`, `*F*`, `foil`, `Set: Name`) turns the card into "not recognized". Edition and foil matching are toggles on the page. A single printing has its own page: `https://www.cardkingdom.com/mtg/innistrad-remastered/delver-of-secrets` (add `-foil` for the foil), on Card Kingdom's own set slugs.

## Proposal
- TCGplayer: `{qty}-{tcgplayer_id}` per card when the printing has an id, `{qty} {name}` otherwise, plus `&productline=Magic`. The id names the exact printing and sidesteps set code and number mismatches.
- Card Kingdom: unchanged, name only; a set marker would break the line.
- Foil: nothing to send until a deck card records a finish, and neither store's list link takes one.
- Check the TCGplayer cart once by hand before shipping, since the honoring was not observed.
