# 408. Stack headers cut long names short, and the cart sits far from the price

State: merged | Owner: halo | PR: pushed to release | Build seen: 6d0bdc52d

## Report
From the composer: group headers in Stacks, on the app and the web, cut long names short ("Planeswalkers (…"). Let the name and count wrap onto a second line instead. And move the cart from the bottom right to right after the price chip. The web's List keeps its far-right cart; the app's List takes the same inline cart as Stacks.

## Fix
halo, CSS only:
- **Stacks headers, web and zwipe.net** (components `deck_page.css`, with the web's copied sheet regenerated) **and app** (`zwiper/assets/main.css`): the name and count wrap (`overflow-wrap: break-word`, no ellipsis). The toggle aligns on the first line's baseline, so the arrow stays beside the name's first line. The cart follows the price chip, with no `margin-left: auto`.
- **App List headers:** the cart follows the price chip, and a wrapped title keeps its arrow on its first line.
- **Web List bands:** unchanged, cart at the far right.

## Checked (headless Chrome, the real sheets)
- App Stacks at 360, the narrowest columns: "Planeswalkers (2)" and "Instants and sorceries (12)" wrap with the arrow on their first line, and the price and cart sit together under them.
- Web Stacks at 1400, six columns: every name reads in full, the one long name wraps, and the cart sits right after the price.
- App List at 393: the cart right after the price, and a wrapped title's arrow on its first line.

## Verification
-
