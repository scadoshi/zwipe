//! What a shared deck's preview image shows, and the SVG that draws it.
//!
//! The image is 1200x630. With command zone art, the art fills the left
//! 500 pixels and the text sits to its right; without it, the text takes
//! the full width. Every number comes from the same zwipe-core functions the
//! share page uses, so the image never disagrees with the page.

use std::fmt::Write;
use std::mem;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::http::contracts::deck::HttpSharedDeck;

use super::palette::Palette;

/// The image's width in pixels, the size every link unfurler shows cleanly.
pub const WIDTH: u32 = 1200;

/// The image's height in pixels.
pub const HEIGHT: u32 = 630;

/// The art panel's width when the deck has command zone art.
const ART_W: i64 = 500;

/// The margin around the text.
const PAD: i64 = 56;

/// `JetBrains Mono`'s advance width as a share of its size: every character
/// is 600 of the font's 1000 units wide.
const ADVANCE: f64 = 0.6;

/// The text family, embedded from `assets/share_preview`.
const TEXT_FONT: &str = "JetBrains Mono";

/// The pip glyphs' family, embedded from `assets/share_preview`.
const MANA_FONT: &str = "Mana";

/// The most command zone cards drawn as art.
const MAX_ART: usize = 2;

/// One command zone card's art.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Art {
    /// The card's art crop on Scryfall's image host.
    pub url: String,
    /// The artist, credited under the text.
    pub artist: Option<String>,
}

/// What the preview shows, read off a shared deck.
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// The format, with the power level when set: "Commander · Upgraded (3)".
    pub format_line: Option<String>,
    /// The deck's name.
    pub name: String,
    /// The command zone's card names: "Led by Tymna the Weaver and Kraum".
    pub led_by: Option<String>,
    /// The deck's colors in WUBRG order; empty is colorless.
    pub colors: Vec<Color>,
    /// The share page's card count.
    pub cards: i64,
    /// The price in US dollars, when any card has one.
    pub price: Option<f64>,
    /// The deck tags' labels.
    pub tags: Vec<String>,
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
        let format_line = match (deck.format, deck.power_level) {
            (Some(format), Some(power)) => Some(format!(
                "{} · {}",
                format.display_name(),
                power.display_name()
            )),
            (Some(format), None) => Some(format.display_name().to_owned()),
            (None, Some(power)) => Some(power.display_name().to_owned()),
            (None, None) => None,
        };
        let zone = deck.zone_cards();
        let names: Vec<&str> = zone.iter().map(|c| c.scryfall_data.name.as_str()).collect();
        let led_by = match names.as_slice() {
            [] => None,
            [only] => Some(format!("Led by {only}")),
            [first @ .., last] => Some(format!("Led by {} and {last}", first.join(", "))),
        };
        let price = deck.price_usd();
        let bars = deck.metrics().mana_curve_bars();
        Self {
            format_line,
            name: deck.name.clone(),
            led_by,
            colors: deck.colors(),
            cards: deck.card_count(),
            price: (price > 0.0).then_some(price),
            tags: deck.tags.iter().map(|t| deck_tag_label(t)).collect(),
            curve: bars.map(|(count, _)| count),
            art: zone.into_iter().filter_map(art).take(MAX_ART).collect(),
        }
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
/// resolves to its bytes. Without art the text takes the full width.
#[must_use]
pub fn svg(preview: &Preview, palette: &Palette, drawn: &[&Art]) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let (w, h) = (i64::from(WIDTH), i64::from(HEIGHT));
    let _ = write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    );
    let _ = write!(
        out,
        r#"<rect width="{w}" height="{h}" fill="{}"/>"#,
        palette.bg
    );
    let x0 = if drawn.is_empty() {
        PAD
    } else {
        ART_W.saturating_add(PAD)
    };
    if !drawn.is_empty() {
        draw_art(&mut out, palette, drawn);
    }
    let text_w = w.saturating_sub(x0).saturating_sub(PAD);
    let mut y = PAD;

    if let Some(line) = &preview.format_line {
        y = y.saturating_add(26);
        text(
            &mut out,
            x0,
            y,
            26,
            400,
            &palette.accent_2,
            &fit(line, text_w, 26),
        );
        y = y.saturating_add(16);
    }

    for line in wrap(&preview.name, text_w, 60, 2) {
        y = y.saturating_add(66);
        text(&mut out, x0, y, 60, 700, &palette.text, &line);
    }

    if let Some(led) = &preview.led_by {
        y = y.saturating_add(46);
        text(
            &mut out,
            x0,
            y,
            26,
            400,
            &palette.muted,
            &fit(led, text_w, 26),
        );
    }

    y = y.saturating_add(28);
    draw_pips(&mut out, palette, x0, y, &preview.colors);
    y = y.saturating_add(PIP).saturating_add(28);

    draw_tags(&mut out, palette, x0, y, text_w, &preview.tags);

    draw_bottom(&mut out, palette, preview, x0, text_w, drawn);
    out.push_str("</svg>");
    out
}

/// The art panel: one image the full height, or two stacked with a rule
/// between them, each cropped to fill its box.
fn draw_art(out: &mut String, palette: &Palette, drawn: &[&Art]) {
    let h = i64::from(HEIGHT);
    let count = i64::try_from(drawn.len()).unwrap_or(1).max(1);
    let each = h.checked_div(count).unwrap_or(h);
    for (i, _) in drawn.iter().enumerate() {
        let top = each.saturating_mul(i64::try_from(i).unwrap_or(0));
        let _ = write!(
            out,
            r#"<image href="art{i}" x="0" y="{top}" width="{ART_W}" height="{each}" preserveAspectRatio="xMidYMid slice"/>"#
        );
        if i > 0 {
            let _ = write!(
                out,
                r#"<rect x="0" y="{}" width="{ART_W}" height="4" fill="{}"/>"#,
                top.saturating_sub(2),
                palette.bg
            );
        }
    }
    let _ = write!(
        out,
        r#"<rect x="{ART_W}" y="0" width="2" height="{h}" fill="{}"/>"#,
        palette.border_soft
    );
}

/// A pip's side in pixels.
const PIP: i64 = 44;

/// The space between pips.
const PIP_GAP: i64 = 10;

/// The deck's colors as the app draws its pips: a rounded square in the
/// theme's mana color, edged and inked with that color darkened. A colorless
/// deck shows the colorless pip.
fn draw_pips(out: &mut String, palette: &Palette, x0: i64, top: i64, colors: &[Color]) {
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
            r#"<rect x="{x}" y="{top}" width="{PIP}" height="{PIP}" rx="13" fill="{fill}" stroke="{ink}" stroke-width="2"/>"#
        );
        let _ = write!(
            out,
            r#"<text x="{}" y="{}" font-family="{MANA_FONT}" font-size="30" fill="{ink}" text-anchor="middle">{glyph}</text>"#,
            x.saturating_add(PIP / 2),
            top.saturating_add(33)
        );
        x = x.saturating_add(PIP).saturating_add(PIP_GAP);
    }
}

/// The colorless pip's fill, mana-font's own.
const COLORLESS_FILL: &str = "#beb9b2";

/// The colorless pip's ink, the app's.
const COLORLESS_INK: &str = "#393835";

/// The deck tags as outlined chips on one line, with "+N" for those that
/// don't fit.
fn draw_tags(out: &mut String, palette: &Palette, x0: i64, top: i64, width: i64, tags: &[String]) {
    const SIZE: i64 = 22;
    const PAD_X: i64 = 14;
    const GAP: i64 = 10;
    const CHIP_H: i64 = 40;
    let right = x0.saturating_add(width);
    let mut x = x0;
    for (i, tag) in tags.iter().enumerate() {
        let left = tags.len().saturating_sub(i).saturating_sub(1);
        let chip_w = text_width(tag, SIZE).saturating_add(PAD_X * 2);
        let more_w = if left > 0 {
            text_width(&format!("+{left}"), SIZE).saturating_add(GAP)
        } else {
            0
        };
        if x.saturating_add(chip_w).saturating_add(more_w) > right {
            let rest = tags.len().saturating_sub(i);
            text(
                out,
                x,
                top.saturating_add(28),
                SIZE,
                400,
                &palette.muted,
                &format!("+{rest}"),
            );
            return;
        }
        let _ = write!(
            out,
            r#"<rect x="{x}" y="{top}" width="{chip_w}" height="{CHIP_H}" rx="10" fill="none" stroke="{}" stroke-width="2"/>"#,
            palette.border
        );
        text(
            out,
            x.saturating_add(PAD_X),
            top.saturating_add(28),
            SIZE,
            400,
            &palette.accent_3,
            tag,
        );
        x = x.saturating_add(chip_w).saturating_add(GAP);
    }
}

/// The card count and price on the left, the mana curve on the right, and
/// under them the art credit and the brand.
fn draw_bottom(
    out: &mut String,
    palette: &Palette,
    preview: &Preview,
    x0: i64,
    width: i64,
    drawn: &[&Art],
) {
    let h = i64::from(HEIGHT);
    let baseline = h.saturating_sub(PAD).saturating_add(4);
    let right = x0.saturating_add(width);

    let cards = if preview.cards == 1 {
        "1 card".to_owned()
    } else {
        format!("{} cards", preview.cards)
    };
    // The count over the price, both left of the curve, so a long price
    // never reaches the curve's labels.
    let stats_y = baseline.saturating_sub(58);
    match preview.price {
        Some(price) => {
            let count_y = stats_y.saturating_sub(44);
            text(out, x0, count_y, 34, 700, &palette.stat, &cards);
            let price = format!("${price:.2}");
            text(out, x0, stats_y, 34, 700, &palette.money, &price);
        }
        None => text(out, x0, stats_y, 34, 700, &palette.stat, &cards),
    }

    draw_curve(out, palette, right, stats_y, &preview.curve);

    let artists: Vec<&str> = drawn.iter().filter_map(|a| a.artist.as_deref()).collect();
    if !artists.is_empty() {
        let credit = format!("Art by {}", artists.join(" and "));
        let room = width
            .saturating_sub(text_width("zwipe.net", 26))
            .saturating_sub(24);
        text(
            out,
            x0,
            baseline,
            20,
            400,
            &palette.muted,
            &fit(&credit, room, 20),
        );
    }
    let _ = write!(
        out,
        r#"<text x="{right}" y="{baseline}" font-family="{TEXT_FONT}" font-size="26" font-weight="700" fill="{}" text-anchor="end">zwipe.net</text>"#,
        palette.accent
    );
}

/// Seven bars, 0 to 6+, ending at `right` and standing on `base`, each as
/// tall as its share of the tallest, with its mana value under it. Nothing
/// when the main board has no nonland card.
fn draw_curve(out: &mut String, palette: &Palette, right: i64, base: i64, curve: &[usize; 7]) {
    const BAR_W: i64 = 30;
    const GAP: i64 = 10;
    const MAX_H: i64 = 72;
    let tallest = curve.iter().copied().max().unwrap_or(0);
    if tallest == 0 {
        return;
    }
    let tallest = i64::try_from(tallest).unwrap_or(i64::MAX);
    let left = right.saturating_sub(
        BAR_W
            .saturating_mul(7)
            .saturating_add(GAP.saturating_mul(6)),
    );
    let floor = base.saturating_sub(24);
    for (i, count) in curve.iter().enumerate() {
        let i64_i = i64::try_from(i).unwrap_or(0);
        let x = left.saturating_add(i64_i.saturating_mul(BAR_W.saturating_add(GAP)));
        let count = i64::try_from(*count).unwrap_or(i64::MAX);
        let bar = MAX_H
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

/// `#rrggbb` as three 0 to 1 channels.
fn hex_rgb(hex: &str) -> Option<[f64; 3]> {
    let digits = hex.strip_prefix('#')?;
    let channel = |at: usize| -> Option<f64> {
        let pair = digits.get(at..at.saturating_add(2))?;
        u8::from_str_radix(pair, 16)
            .ok()
            .map(|v| f64::from(v) / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?])
}

/// Three 0 to 1 channels as `#rrggbb`, clamped.
fn rgb_hex(rgb: [f64; 3]) -> String {
    let byte = |v: f64| -> u8 {
        let scaled = (v.clamp(0.0, 1.0) * 255.0).round();
        u8::try_from(int_of(scaled)).unwrap_or(u8::MAX)
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(rgb[0]),
        byte(rgb[1]),
        byte(rgb[2])
    )
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
