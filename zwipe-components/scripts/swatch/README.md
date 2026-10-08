# Theme swatches

Renders every block of `../../assets/themes.css` as a card: background, text, link, selected state, the three accents, the three status colors, and the six palette slots with tag chips drawn in them. Local only; never deployed.

```bash
python3 -I gen.py ../../assets/themes.css - swatch-dark.html dark
python3 -I gen.py ../../assets/themes.css - swatch-light.html light
```

Open the HTML in a browser, or screenshot it headless. The footer of each card names the closest pair of the six and whether any slot sits near a status color (informational; see `context/plans/palette.md`). `colors.py` holds the OKLab, contrast and color-vision-deficiency math, the same checks `tests/themes_palette.rs` enforces.
