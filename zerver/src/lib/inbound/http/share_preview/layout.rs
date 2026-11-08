//! What a shared deck's preview image shows, and the SVG that draws it.
//!
//! The image is 1200x630 and follows the site's own share image: the app's
//! grid tiles on the sink, and one panel over them. Inside the panel the deck
//! reads like a row of the app's deck list: the name, then the color pips and
//! the same tags the list shows, with the mana curve under them and a footer
//! holding the art credit and the brand. With command zone art, the art fills
//! the panel's left side. Every number comes from the same zwipe-core
//! functions the share page uses, so the image never disagrees with the page.

use std::fmt::Write;
use std::mem;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::http::contracts::deck::HttpSharedDeck;

use super::palette::{Palette, hex_rgb, mix, rgb_hex};

/// The image's width in pixels, the size every link unfurler shows cleanly.
pub const WIDTH: u32 = 1200;

/// The image's height in pixels.
pub const HEIGHT: u32 = 630;

/// The panel's left edge, the site's share image's margin.
const PANEL_X: i64 = 60;

/// The panel's width, leaving the same margin on the right.
const PANEL_W: i64 = 1080;

/// The panel's height.
const PANEL_H: i64 = 504;

/// The panel's top edge, centering it.
const PANEL_Y: i64 = 63;

/// The panel's corner radius.
const PANEL_R: i64 = 24;

/// The art's width at the panel's left, above the footer, when the deck has
/// command zone art.
const ART_W: i64 = 380;

/// The space between the panel's edges and its text, sideways.
const PAD_X: i64 = 48;

/// The space between the panel's top and the name.
const PAD_Y: i64 = 40;

/// The footer's height: the strip under the rule holding the art credit and
/// the brand.
const FOOTER_H: i64 = 84;

/// The name's size.
const NAME_SIZE: i64 = 48;

/// A tag's height, which the pips match.
const CHIP_H: i64 = 40;

/// A tag's text size.
const CHIP_SIZE: i64 = 21;

/// The space between a tag's edge and its text.
const CHIP_PAD: i64 = 14;

/// The space between tags on a row.
const CHIP_GAP: i64 = 10;

/// The space between rows of tags.
const ROW_GAP: i64 = 12;

/// A tag's baseline below its top.
const CHIP_BASELINE: i64 = 27;

/// `JetBrains Mono`'s advance width as a share of its size: every character
/// is 600 of the font's 1000 units wide.
const ADVANCE: f64 = 0.6;

/// The text family, embedded from `assets/share_preview`.
const TEXT_FONT: &str = "JetBrains Mono";

/// The pip glyphs' family, embedded from `assets/share_preview`.
const MANA_FONT: &str = "Mana";

/// The most command zone cards drawn as art.
const MAX_ART: usize = 2;

/// The grid tiles' pitch, orangui's `--grid-size`.
const CELL: i64 = 28;

/// The tiles' opacities, orangui's `grid-cells.js` `SHADES`.
const SHADES: [f64; 5] = [0.015, 0.03, 0.05, 0.075, 0.1];

/// `grid-cells.js`'s seed, so the image shows the app's own field.
const SEED: u32 = 0x9e37_79b9;

/// One command zone card's art.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Art {
    /// The card's art crop on Scryfall's image host.
    pub url: String,
    /// The artist, credited in the footer.
    pub artist: Option<String>,
}

/// A tag's color, named after the deck list's chip class that sets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    /// `stat-chip`: the card count.
    Accent,
    /// `stat-chip-p3`: the format and the command zone's cards.
    Palette3,
    /// `stat-chip-p5`: the power level.
    Palette5,
    /// `stat-chip-money`: the price.
    Money,
    /// `stat-chip-muted`: the deck tags and the art credit's label.
    Muted,
}

/// One tag: a label drawn as the deck list's chip in a tone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// The text.
    pub label: String,
    /// The color.
    pub tone: Tone,
}

impl Tag {
    fn new(label: impl Into<String>, tone: Tone) -> Self {
        Self {
            label: label.into(),
            tone,
        }
    }
}

/// What the preview shows, read off a shared deck.
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// The deck's name.
    pub name: String,
    /// The deck's colors in WUBRG order; empty is colorless.
    pub colors: Vec<Color>,
    /// The tags after the pips, in the deck list's order: the card count, the
    /// format, the power level, the price, each command zone card and the
    /// deck tags. The price is left out when no card has one.
    pub tags: Vec<Tag>,
    /// Main board cards per mana value, 0 to 5 and 6+, lands left out.
    pub curve: [usize; 7],
    /// Up to two command zone cards' art: the commander, then its partner,
    /// background or signature spell.
    pub art: Vec<Art>,
}

impl Preview {
    /// The preview of `deck`.
    #[must_use]
    pub fn of(deck: &HttpSharedDeck) -> Self {
        let zone = deck.zone_cards();
        let mut tags = vec![Tag::new(cards_label(deck.card_count()), Tone::Accent)];
        if let Some(format) = deck.format {
            tags.push(Tag::new(format.display_name(), Tone::Palette3));
        }
        if let Some(power) = deck.power_level {
            tags.push(Tag::new(power.display_name(), Tone::Palette5));
        }
        let price = deck.price_usd();
        if price > 0.0 {
            tags.push(Tag::new(format!("${price:.2}"), Tone::Money));
        }
        tags.extend(
            zone.iter()
                .map(|card| Tag::new(card.scryfall_data.name.as_str(), Tone::Palette3)),
        );
        tags.extend(
            deck.tags
                .iter()
                .map(|slug| Tag::new(deck_tag_label(slug), Tone::Muted)),
        );
        let bars = deck.metrics().mana_curve_bars();
        Self {
            name: deck.name.clone(),
            colors: deck.colors(),
            tags,
            curve: bars.map(|(count, _)| count),
            art: zone.into_iter().filter_map(art).take(MAX_ART).collect(),
        }
    }
}

/// "1 card" or "N cards", as the deck list words its count.
fn cards_label(count: i64) -> String {
    if count == 1 {
        "1 card".to_owned()
    } else {
        format!("{count} cards")
    }
}

/// `card`'s art, when it has an art crop.
fn art(card: &Card) -> Option<Art> {
    let data = &card.scryfall_data;
    let url = data.art_crop_url()?.to_owned();
    let face_artist = data
        .card_faces
        .as_ref()
        .and_then(|faces| faces.first())
        .and_then(|face| face.artist.clone());
    Some(Art {
        url,
        artist: data.artist.clone().or(face_artist),
    })
}

/// The SVG for `preview` in `palette`. `drawn` is the art that loaded, in
/// order; the SVG refers to each by `href="art{i}"`, which the renderer
/// resolves to its bytes. Without art the text takes the panel's width.
#[must_use]
pub fn svg(preview: &Preview, palette: &Palette, drawn: &[&Art]) -> String {
    let mut out = String::with_capacity(64 * 1024);
    let (w, h) = (i64::from(WIDTH), i64::from(HEIGHT));
    let panel = format!(
        r#"x="{PANEL_X}" y="{PANEL_Y}" width="{PANEL_W}" height="{PANEL_H}" rx="{PANEL_R}""#
    );
    let _ = write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    );
    let _ = write!(
        out,
        r#"<defs><clipPath id="panel"><rect {panel}/></clipPath></defs>"#
    );
    draw_grid(&mut out, palette);
    draw_shadow(&mut out, palette);
    let _ = write!(out, r#"<rect {panel} fill="{}"/>"#, palette.bg);

    let text_left = if drawn.is_empty() {
        PANEL_X
    } else {
        draw_art(&mut out, palette, drawn);
        PANEL_X.saturating_add(ART_W)
    };
    let x0 = text_left.saturating_add(PAD_X);
    let right = PANEL_X.saturating_add(PANEL_W).saturating_sub(PAD_X);
    let text_w = right.saturating_sub(x0);
    let rule_y = PANEL_Y.saturating_add(PANEL_H).saturating_sub(FOOTER_H);

    let mut y = PANEL_Y.saturating_add(PAD_Y);
    for (i, line) in wrap(&preview.name, text_w, NAME_SIZE, 2).iter().enumerate() {
        y = y.saturating_add(if i == 0 { 42 } else { 58 });
        text(&mut out, x0, y, NAME_SIZE, 400, &palette.text, line);
    }

    let curve_base = rule_y.saturating_sub(18);
    let has_curve = preview.curve.iter().any(|count| *count > 0);
    let area = Area {
        left: x0,
        top: y.saturating_add(24),
        right,
        beside_curve: if has_curve {
            curve_base.saturating_sub(CURVE_TOP).saturating_sub(14)
        } else {
            i64::MAX
        },
        curve_left: right.saturating_sub(CURVE_W).saturating_sub(24),
        floor: rule_y.saturating_sub(20),
    };
    draw_tags(&mut out, palette, &area, &preview.colors, &preview.tags);
    draw_curve(&mut out, palette, right, curve_base, &preview.curve);

    let _ = write!(
        out,
        r#"<rect x="{PANEL_X}" y="{rule_y}" width="{PANEL_W}" height="2" fill="{}"/>"#,
        palette.border_soft
    );
    let footer_x = PANEL_X.saturating_add(PAD_X);
    draw_footer(&mut out, palette, footer_x, right, rule_y, drawn);
    let _ = write!(
        out,
        r#"<rect {panel} fill="none" stroke="{}" stroke-width="2"/>"#,
        palette.border_soft
    );
    out.push_str("</svg>");
    out
}

/// The panel's shadow, the app's `--shadow-md` (4px down, 12px soft): rings
/// of black around the panel, each wider and fainter, which reads as a blur
/// without the cost of a filter. Lighter under a light theme.
fn draw_shadow(out: &mut String, palette: &Palette) {
    let strength = if palette.light { 0.18 } else { 0.5 };
    let rings: i64 = 6;
    out.push_str(r##"<g fill="#000000">"##);
    for ring in (1..=rings).rev() {
        let grow = ring.saturating_mul(2);
        let alpha = strength / f64_of(rings) * 1.4;
        let _ = write!(
            out,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill-opacity="{alpha:.3}"/>"#,
            PANEL_X.saturating_sub(grow),
            PANEL_Y.saturating_add(4).saturating_sub(grow),
            PANEL_W.saturating_add(grow.saturating_mul(2)),
            PANEL_H.saturating_add(grow.saturating_mul(2)),
            PANEL_R.saturating_add(grow)
        );
    }
    out.push_str("</g>");
}

/// The app's backdrop: the sink, and on it the grid tiles as orangui's
/// `grid-cells.js` draws its settled field, brightest at the top and fading
/// out down the image. Tiles the panel covers are left out.
fn draw_grid(out: &mut String, palette: &Palette) {
    let (w, h) = (i64::from(WIDTH), i64::from(HEIGHT));
    let _ = write!(
        out,
        r#"<rect width="{w}" height="{h}" fill="{}"/><g fill="{}">"#,
        palette.sink, palette.text
    );
    let side = CELL.saturating_sub(2);
    let radius = f64_of(side) * 0.18;
    let cols = w.saturating_add(CELL - 1).checked_div(CELL).unwrap_or(0);
    let rows = h.saturating_add(CELL - 1).checked_div(CELL).unwrap_or(0);
    let mut seed = SEED;
    for row in 0..rows {
        let y = row.saturating_mul(CELL);
        let fade = grid_fade(f64_of(y) / f64_of(h));
        for col in 0..cols {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let roll = usize::try_from(seed >> 8).unwrap_or(0);
            let shade = SHADES
                .get(roll.checked_rem(SHADES.len()).unwrap_or(0))
                .copied()
                .unwrap_or(0.0);
            let alpha = shade * fade;
            let x = col.saturating_mul(CELL);
            if alpha < 0.002 || under_panel(x, y) {
                continue;
            }
            let _ = write!(
                out,
                r#"<rect x="{}" y="{}" width="{side}" height="{side}" rx="{radius:.2}" fill-opacity="{alpha:.4}"/>"#,
                x.saturating_add(1),
                y.saturating_add(1)
            );
        }
    }
    out.push_str("</g>");
}

/// The tiles' opacity at `t` of the way down the image, orangui's mask: full
/// at the top, 0.45 at 45%, gone at the bottom.
fn grid_fade(t: f64) -> f64 {
    if t < 0.45 {
        1.0 - (t / 0.45) * 0.55
    } else {
        0.45 * (1.0 - (t - 0.45) / 0.55)
    }
}

/// Whether the tile at (`x`, `y`) sits wholly under the panel, clear of its
/// rounded corners.
fn under_panel(x: i64, y: i64) -> bool {
    let inside = |at: i64, start: i64, len: i64| {
        at >= start.saturating_add(PANEL_R)
            && at.saturating_add(CELL) <= start.saturating_add(len).saturating_sub(PANEL_R)
    };
    inside(x, PANEL_X, PANEL_W) && inside(y, PANEL_Y, PANEL_H)
}

/// The art on the panel's left above the footer: one image, or two stacked
/// with a rule between them, each cropped to fill its box and clipped to the
/// panel's corners.
fn draw_art(out: &mut String, palette: &Palette, drawn: &[&Art]) {
    let count = i64::try_from(drawn.len()).unwrap_or(1).max(1);
    let height = PANEL_H.saturating_sub(FOOTER_H);
    let each = height.checked_div(count).unwrap_or(height);
    out.push_str(r#"<g clip-path="url(#panel)">"#);
    for (i, _) in drawn.iter().enumerate() {
        let top = PANEL_Y.saturating_add(each.saturating_mul(i64::try_from(i).unwrap_or(0)));
        let _ = write!(
            out,
            r#"<image href="art{i}" x="{PANEL_X}" y="{top}" width="{ART_W}" height="{each}" preserveAspectRatio="xMidYMid slice"/>"#
        );
        if i > 0 {
            let _ = write!(
                out,
                r#"<rect x="{PANEL_X}" y="{}" width="{ART_W}" height="4" fill="{}"/>"#,
                top.saturating_sub(2),
                palette.bg
            );
        }
    }
    out.push_str("</g>");
    let _ = write!(
        out,
        r#"<rect x="{}" y="{PANEL_Y}" width="2" height="{height}" fill="{}"/>"#,
        PANEL_X.saturating_add(ART_W).saturating_sub(1),
        palette.border_soft
    );
}

/// The box the pips and tags flow in: rows start at `left` and `top`, wrap
/// at `right`, and stop above `floor`. A row reaching below `beside_curve`
/// wraps at `curve_left` instead, clear of the mana curve.
#[derive(Debug, Clone, Copy)]
struct Area {
    left: i64,
    top: i64,
    right: i64,
    beside_curve: i64,
    curve_left: i64,
    floor: i64,
}

impl Area {
    /// Where the row starting at `row` wraps.
    fn right_at(&self, row: i64) -> i64 {
        if row.saturating_add(CHIP_H) > self.beside_curve {
            self.curve_left
        } else {
            self.right
        }
    }
}

/// The pips, then the tags, flowing in rows across `area` like the deck
/// list's chip row. When the rows run out, a "+N" counts the tags left.
fn draw_tags(out: &mut String, palette: &Palette, area: &Area, colors: &[Color], tags: &[Tag]) {
    let mut row = area.top;
    let mut x = draw_pips(out, palette, area.left, row, colors);
    let width = area.right.saturating_sub(area.left);
    let last_row = |row: i64| {
        row.saturating_add(CHIP_H)
            .saturating_add(ROW_GAP)
            .saturating_add(CHIP_H)
            > area.floor
    };
    for (i, tag) in tags.iter().enumerate() {
        let label = fit(&tag.label, width.saturating_sub(CHIP_PAD * 2), CHIP_SIZE);
        let chip_w = text_width(&label, CHIP_SIZE).saturating_add(CHIP_PAD * 2);
        let after = tags.len().saturating_sub(i).saturating_sub(1);
        let more_w = text_width(&format!("+{after}"), CHIP_SIZE).saturating_add(CHIP_GAP);
        let needs = |row: i64| {
            if last_row(row) && after > 0 {
                chip_w.saturating_add(more_w)
            } else {
                chip_w
            }
        };
        if x > area.left && x.saturating_add(needs(row)) > area.right_at(row) && !last_row(row) {
            row = row.saturating_add(CHIP_H).saturating_add(ROW_GAP);
            x = area.left;
        }
        if x.saturating_add(needs(row)) > area.right_at(row) {
            let rest = tags.len().saturating_sub(i);
            let baseline = row.saturating_add(CHIP_BASELINE);
            let more = format!("+{rest}");
            text(out, x, baseline, CHIP_SIZE, 400, &palette.ink.muted, &more);
            return;
        }
        chip(out, palette, x, row, &label, tag.tone);
        x = x.saturating_add(chip_w).saturating_add(CHIP_GAP);
    }
}

/// One tag as the deck list's chip: its tone's ink on a 12% tint of the tone,
/// edged with the tone at 35% (30% for a muted tag). Returns its width.
fn chip(out: &mut String, palette: &Palette, x: i64, top: i64, label: &str, tone: Tone) -> i64 {
    let (color, ink, edge) = match tone {
        Tone::Accent => (&palette.accent, &palette.ink.accent, 0.35),
        Tone::Palette3 => (&palette.palette_3, &palette.ink.palette_3, 0.35),
        Tone::Palette5 => (&palette.palette_5, &palette.ink.palette_5, 0.35),
        Tone::Money => (&palette.money, &palette.ink.money, 0.35),
        Tone::Muted => (&palette.muted, &palette.ink.muted, 0.30),
    };
    let tint = mix(&palette.bg, color, 0.12).unwrap_or_else(|| palette.bg.clone());
    let border = mix(&palette.bg, color, edge).unwrap_or_else(|| color.clone());
    let chip_w = text_width(label, CHIP_SIZE).saturating_add(CHIP_PAD * 2);
    let _ = write!(
        out,
        r#"<rect x="{x}" y="{top}" width="{chip_w}" height="{CHIP_H}" rx="12" fill="{tint}" stroke="{border}" stroke-width="2"/>"#
    );
    text(
        out,
        x.saturating_add(CHIP_PAD),
        top.saturating_add(CHIP_BASELINE),
        CHIP_SIZE,
        400,
        ink,
        label,
    );
    chip_w
}

/// The space between pips.
const PIP_GAP: i64 = 6;

/// The deck's colors as the app draws its pips, at a tag's height: a rounded
/// square in the theme's mana color, edged and inked with that color
/// darkened. A colorless deck shows the colorless pip. Returns where the
/// next tag starts.
fn draw_pips(out: &mut String, palette: &Palette, x0: i64, top: i64, colors: &[Color]) -> i64 {
    let pips: Vec<(&str, char)> = if colors.is_empty() {
        vec![(COLORLESS_FILL, '\u{e904}')]
    } else {
        colors
            .iter()
            .map(|c| {
                let (i, glyph) = match c {
                    Color::White => (0, '\u{e600}'),
                    Color::Blue => (1, '\u{e601}'),
                    Color::Black => (2, '\u{e602}'),
                    Color::Red => (3, '\u{e603}'),
                    Color::Green => (4, '\u{e604}'),
                };
                (
                    palette.mana.get(i).map_or(COLORLESS_FILL, String::as_str),
                    glyph,
                )
            })
            .collect()
    };
    let mut x = x0;
    for (fill, glyph) in pips {
        let ink = if fill == COLORLESS_FILL {
            COLORLESS_INK.to_owned()
        } else {
            pip_ink(fill)
        };
        let _ = write!(
            out,
            r#"<rect x="{x}" y="{top}" width="{CHIP_H}" height="{CHIP_H}" rx="12" fill="{fill}" stroke="{ink}" stroke-width="2"/>"#
        );
        let _ = write!(
            out,
            r#"<text x="{}" y="{}" font-family="{MANA_FONT}" font-size="27" fill="{ink}" text-anchor="middle">{glyph}</text>"#,
            x.saturating_add(CHIP_H / 2),
            top.saturating_add(30)
        );
        x = x.saturating_add(CHIP_H).saturating_add(PIP_GAP);
    }
    x.saturating_sub(PIP_GAP).saturating_add(CHIP_GAP)
}

/// The colorless pip's fill, mana-font's own.
const COLORLESS_FILL: &str = "#beb9b2";

/// The colorless pip's ink, the app's.
const COLORLESS_INK: &str = "#393835";

/// The footer under the rule: an "Art" label tag followed by the artists of
/// the art drawn on the left, and the brand on the right.
fn draw_footer(
    out: &mut String,
    palette: &Palette,
    x0: i64,
    right: i64,
    rule_y: i64,
    drawn: &[&Art],
) {
    const BRAND: &str = "zwipe.net";
    const BRAND_SIZE: i64 = 28;
    const BRAND_SPACING: i64 = 4;
    let bottom = PANEL_Y.saturating_add(PANEL_H);
    let top = rule_y.saturating_add(
        bottom
            .saturating_sub(rule_y)
            .saturating_sub(CHIP_H)
            .checked_div(2)
            .unwrap_or(0),
    );
    let baseline = top.saturating_add(CHIP_BASELINE);
    let _ = write!(
        out,
        r#"<text x="{right}" y="{}" font-family="{TEXT_FONT}" font-size="{BRAND_SIZE}" font-weight="700" letter-spacing="{BRAND_SPACING}" fill="{}" text-anchor="end">{BRAND}</text>"#,
        baseline.saturating_add(2),
        palette.palette_6
    );
    let artists: Vec<&str> = drawn.iter().filter_map(|a| a.artist.as_deref()).collect();
    if artists.is_empty() {
        return;
    }
    let label_w = chip(out, palette, x0, top, "Art", Tone::Muted);
    let names_x = x0.saturating_add(label_w).saturating_add(CHIP_GAP + 4);
    let brand_w = text_width(BRAND, BRAND_SIZE).saturating_add(BRAND_SPACING.saturating_mul(9));
    let room = right
        .saturating_sub(brand_w)
        .saturating_sub(24)
        .saturating_sub(names_x);
    let names = fit(&artists.join(" and "), room, CHIP_SIZE);
    text(
        out,
        names_x,
        baseline,
        CHIP_SIZE,
        400,
        &palette.text,
        &names,
    );
}

/// A curve bar's width.
const BAR_W: i64 = 30;

/// The space between curve bars.
const BAR_GAP: i64 = 10;

/// The curve's width: seven bars and the six gaps between them.
const CURVE_W: i64 = BAR_W * 7 + BAR_GAP * 6;

/// The tallest bar's height.
const CURVE_MAX_H: i64 = 64;

/// How far above its labels' baseline the curve reaches.
const CURVE_TOP: i64 = CURVE_MAX_H + 24;

/// Seven bars, 0 to 6+, ending at `right` with their labels on `base`, each
/// as tall as its share of the tallest. Nothing when the main board has no
/// nonland card.
fn draw_curve(out: &mut String, palette: &Palette, right: i64, base: i64, curve: &[usize; 7]) {
    let tallest = curve.iter().copied().max().unwrap_or(0);
    if tallest == 0 {
        return;
    }
    let tallest = i64::try_from(tallest).unwrap_or(i64::MAX);
    let left = right.saturating_sub(CURVE_W);
    let floor = base.saturating_sub(24);
    for (i, count) in curve.iter().enumerate() {
        let i64_i = i64::try_from(i).unwrap_or(0);
        let x = left.saturating_add(i64_i.saturating_mul(BAR_W.saturating_add(BAR_GAP)));
        let count = i64::try_from(*count).unwrap_or(i64::MAX);
        let bar = CURVE_MAX_H
            .saturating_mul(count)
            .checked_div(tallest)
            .unwrap_or(0)
            .max(if count > 0 { 4 } else { 0 });
        if bar > 0 {
            let _ = write!(
                out,
                r#"<rect x="{x}" y="{}" width="{BAR_W}" height="{bar}" rx="4" fill="{}"/>"#,
                floor.saturating_sub(bar),
                palette.accent
            );
        }
        let label = if i == 6 {
            "6+".to_owned()
        } else {
            i.to_string()
        };
        let _ = write!(
            out,
            r#"<text x="{}" y="{base}" font-family="{TEXT_FONT}" font-size="18" fill="{}" text-anchor="middle">{label}</text>"#,
            x.saturating_add(BAR_W / 2),
            palette.muted
        );
    }
    let _ = write!(
        out,
        r#"<rect x="{left}" y="{floor}" width="{}" height="2" fill="{}"/>"#,
        right.saturating_sub(left),
        palette.border_soft
    );
}

/// One line of text at (`x`, `y`), its baseline, XML-escaped.
fn text(out: &mut String, x: i64, y: i64, size: i64, weight: u16, fill: &str, content: &str) {
    let _ = write!(
        out,
        r#"<text x="{x}" y="{y}" font-family="{TEXT_FONT}" font-size="{size}" font-weight="{weight}" fill="{fill}">{}</text>"#,
        escape(content)
    );
}

/// `content` with the characters XML reserves escaped.
fn escape(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for c in content.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// How many characters of `size` fit in `width` pixels.
fn chars_fitting(width: i64, size: i64) -> usize {
    let advance = ADVANCE * f64_of(size);
    if advance <= 0.0 {
        return 0;
    }
    let fit = (f64_of(width) / advance).floor();
    usize::try_from(int_of(fit)).unwrap_or(0)
}

/// `content`'s width in pixels at `size`.
fn text_width(content: &str, size: i64) -> i64 {
    let chars = i64::try_from(content.chars().count()).unwrap_or(i64::MAX);
    int_of((ADVANCE * f64_of(size) * f64_of(chars)).ceil())
}

/// `content` cut to fit `width` at `size`, ending in "…" when cut.
fn fit(content: &str, width: i64, size: i64) -> String {
    let max = chars_fitting(width, size);
    if content.chars().count() <= max {
        return content.to_owned();
    }
    let kept: String = content.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// `content` word-wrapped to lines that fit `width` at `size`, at most
/// `lines` of them, the last ending in "…" when the text runs on. A word
/// longer than a line is split.
fn wrap(content: &str, width: i64, size: i64, lines: usize) -> Vec<String> {
    let max = chars_fitting(width, size).max(1);
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in content.split_whitespace() {
        let mut word = word.to_owned();
        while word.chars().count() > max {
            if !line.is_empty() {
                out.push(mem::take(&mut line));
            }
            let head: String = word.chars().take(max).collect();
            word = word.chars().skip(max).collect();
            out.push(head);
        }
        let joined = if line.is_empty() {
            word.clone()
        } else {
            format!("{line} {word}")
        };
        if joined.chars().count() <= max {
            line = joined;
        } else {
            out.push(mem::replace(&mut line, word));
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    if out.len() > lines {
        let rest = out.split_off(lines.saturating_sub(1)).join(" ");
        let kept: String = rest.chars().take(max.saturating_sub(1)).collect();
        out.push(format!("{}…", kept.trim_end()));
    }
    out
}

/// The ink of a pip filled with `fill`, as the app's stylesheet derives it:
/// the fill mixed 42% into black in Oklab, or, for a fill darker than Oklab
/// lightness 0.55, a light ink so the glyph keeps its contrast.
fn pip_ink(fill: &str) -> String {
    let Some(rgb) = hex_rgb(fill) else {
        return COLORLESS_INK.to_owned();
    };
    let [l, a, b] = oklab(rgb);
    let dark = l < 0.55;
    let (l2, scale) = if dark {
        (l * 0.42 + (0.7 - l * 0.12), 0.30)
    } else {
        (l * 0.42, 0.42)
    };
    rgb_hex(srgb([l2, a * scale, b * scale]))
}

/// sRGB to Oklab, per Björn Ottosson's reference.
fn oklab([red, green, blue]: [f64; 3]) -> [f64; 3] {
    let linear = |c: f64| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (red, green, blue) = (linear(red), linear(green), linear(blue));
    let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
    let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
    let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
    [
        0.210_454_255_3 * long + 0.793_617_785_0 * medium - 0.004_072_046_8 * short,
        1.977_998_495_1 * long - 2.428_592_205_0 * medium + 0.450_593_709_9 * short,
        0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766_0 * short,
    ]
}

/// Oklab to sRGB, per Björn Ottosson's reference.
fn srgb([lightness, green_red, blue_yellow]: [f64; 3]) -> [f64; 3] {
    let long = (lightness + 0.396_337_777_4 * green_red + 0.215_803_757_3 * blue_yellow).powi(3);
    let medium = (lightness - 0.105_561_345_8 * green_red - 0.063_854_172_8 * blue_yellow).powi(3);
    let short = (lightness - 0.089_484_177_5 * green_red - 1.291_485_548_0 * blue_yellow).powi(3);
    let gamma = |c: f64| {
        if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };
    [
        gamma(4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short),
        gamma(-1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short),
        gamma(-0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701_0 * short),
    ]
}

/// `v` as a float. Pixel values here are far inside f64's exact range.
#[expect(
    clippy::cast_precision_loss,
    reason = "pixel sizes and character counts stay far below 2^52"
)]
#[expect(
    clippy::as_conversions,
    reason = "i64 to f64 has no From; the values are small"
)]
fn f64_of(v: i64) -> f64 {
    v as f64
}

/// `v`, already rounded, as an integer; saturates at the ends.
#[expect(
    clippy::cast_possible_truncation,
    reason = "callers round first and the values are small"
)]
#[expect(
    clippy::as_conversions,
    reason = "f64 to i64 has no TryFrom; `as` saturates"
)]
fn int_of(v: f64) -> i64 {
    v as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_name_wraps_to_two_lines_and_ends_in_an_ellipsis() {
        let name = "The Extremely Long Deck Name That Will Not Fit On Two Lines Of The Preview Card At All";
        let lines = wrap(name, 584, 60, 2);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[1].ends_with('…'), "{lines:?}");
        for line in &lines {
            assert!(line.chars().count() <= chars_fitting(584, 60), "{line}");
        }
        assert_eq!(wrap("Krenko Tokens", 584, 60, 2), vec!["Krenko Tokens"]);
    }

    #[test]
    fn a_word_longer_than_a_line_is_split() {
        let lines = wrap(&"x".repeat(40), 300, 60, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].chars().count() <= chars_fitting(300, 60));
    }

    #[test]
    fn text_is_escaped_for_xml() {
        assert_eq!(
            escape(r#"<Tom & "Jerry's">"#),
            "&lt;Tom &amp; &quot;Jerry&apos;s&quot;&gt;"
        );
    }

    #[test]
    fn pip_ink_darkens_a_light_fill_and_lightens_a_dark_one() {
        let light = hex_rgb(&pip_ink("#ebdbb2")).unwrap();
        assert!(light.iter().all(|c| *c < 0.5), "{light:?}");
        let dark = hex_rgb(&pip_ink("#282828")).unwrap();
        assert!(dark.iter().all(|c| *c > 0.4), "{dark:?}");
    }

    #[test]
    fn oklab_round_trips() {
        for hex in ["#83a598", "#fb4934", "#ebdbb2", "#000000", "#ffffff"] {
            assert_eq!(rgb_hex(srgb(oklab(hex_rgb(hex).unwrap()))), hex);
        }
    }
}
