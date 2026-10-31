//! Playstyle: the title, then one scrolled column of the profile sections, the Show row, the suggestions and the staples, then the action bar.
//!
//! The column scrolls so the focused piece is in view, in Activity's grammar: a marker beside the focused line, section bodies indented under their headers.

use std::mem;
use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use uuid::Uuid;
use zwipe_core::domain::card::card_role;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderSuggestion, HttpStaple, HttpSuggestionReasons,
};

use super::{
    COLD_START, Focus, LIST_TITLE, LOAD_FAILED, NO_STAPLES, NO_SUGGESTIONS, Overlay, PlaystyleView,
    SHARES_TAGS, STAPLES_TITLE, Section, WHY,
};
use crate::domain::activity::Bar;
use crate::domain::color_map::{ColorMapCounts, ColorMapModel};
use crate::domain::playstyle;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::activity::draw as activity_draw;
use crate::inbound::screens::activity::draw::{Body, GHOST, GUTTER, INDENT};
use crate::inbound::screens::home;
use crate::inbound::screens::home::draw as home_draw;
use crate::inbound::widgets::card_rows::{CardRow, OTHER_TAGS};
use crate::inbound::widgets::{action_bar, charts, color_map, header, mana, otag};

/// The tallest the color map draws, in rows, as the app's `14rem`.
const MAP_ROWS: u16 = 14;

/// A number column of the tags table, in cells.
const COLUMN_WIDTH: usize = 10;

/// The muted rule over a suggestion's reasons, in cells.
const RULE_WIDTH: usize = 24;

/// Ghost rows in the list while loading, as the app's skeleton.
const GHOST_ROWS: usize = 6;

/// The Show row's chip ghosts while loading: small, small, medium, medium, small.
const GHOST_CHIPS: [usize; 5] = [4, 4, 7, 7, 4];

/// The disclosure arrow on the reasons chip, folded and unfolded.
const FOLDED: &str = "▸";
const UNFOLDED: &str = "▾";

/// How a suggestion's reasons show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reasons {
    /// Under a closed "Why this commander?" chip, as a list row keeps them.
    Folded,
    /// Under an open chip.
    Unfolded,
    /// Open under a plain heading, as the commander swipe shows the card on top.
    Heading,
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &PlaystyleView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let bar = action_bar::line(palette, &key_hints(view));
    let bar_height = action_bar::height(&bar, area.width);
    let [title, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(bar_height),
    ])
    .areas(area);
    header::draw(f, palette, title, super::TITLE, header::Corner::Hint);
    if view.is_loading {
        activity_draw::draw_lines(f, skeleton(palette, view, body.width), (0, 0), body);
    } else if view.load_failed {
        activity_draw::draw_note(f, palette, LOAD_FAILED, body);
    } else if view.cold_start() {
        activity_draw::draw_note(f, palette, COLD_START, body);
    } else {
        let body_lines = loaded_lines(shared, view, body.width);
        activity_draw::draw_lines(f, body_lines.lines, body_lines.focus, body);
    }
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    match &view.overlay {
        Some(Overlay::Info(info)) => info.draw(f, palette),
        Some(Overlay::More(sheet)) => sheet.draw(f, palette, f.area()),
        Some(Overlay::Clear(dialog)) => {
            super::keys::more_sheet(palette).draw(f, palette, f.area());
            dialog.draw(f, palette);
        }
        None => {}
    }
}

/// The keys for what has the focus.
fn key_hints(view: &PlaystyleView) -> Vec<(&'static str, &'static str)> {
    let mut keys = Vec::new();
    if view.ready() {
        keys.push(("j/k", "Move"));
        match view.focus {
            Focus::Header(section) => {
                let label = if view.is_open(section) {
                    "Close"
                } else {
                    "Open"
                };
                keys.push(("Enter", label));
            }
            Focus::Role(index) => {
                if view
                    .role_bars()
                    .get(index)
                    .is_some_and(|bar| bar.revealable)
                {
                    keys.push(("Enter", "Reveal"));
                }
                if view.open_role.is_some() {
                    keys.push(("Tab", "Tags"));
                }
                if view.role_tags.open_tag().is_some() {
                    keys.push(("x", "Examples"));
                }
            }
            Focus::TagHead => keys.push(("h/l", "Column")),
            Focus::Tag(_) => {
                keys.push(("Enter", "Reveal"));
                if view.open_tag.is_some() {
                    keys.push(("x", "Examples"));
                }
            }
            Focus::Show | Focus::Types | Focus::Colors => {
                keys.push(("h/l", "Chip"));
                keys.push(("Enter", "Pick"));
            }
            Focus::List | Focus::Staples => keys.push(("Enter", "Expand")),
        }
        if view.topic().is_some() {
            keys.push(("?", "About"));
        }
    }
    if !view.cold_start() {
        keys.push(("s", "Swipe"));
    }
    keys.push(("M", "More"));
    keys.push(("H", "Hint"));
    keys.push(("Esc", "Back"));
    keys
}

/// A chip on a chip row: its spans, and whether it is the picked one.
pub(crate) struct ChipContent {
    pub(crate) spans: Vec<Span<'static>>,
    pub(crate) picked: bool,
}

impl ChipContent {
    pub(crate) fn text(label: &str, picked: bool) -> Self {
        Self {
            spans: vec![Span::raw(label.to_owned())],
            picked,
        }
    }
}

/// A chip row: the label chip, then each chip padded, the picked one in the accent reversed and the one under the cursor underlined while `focused`.
pub(crate) fn chip_line(
    palette: &Palette,
    label: &str,
    chips: Vec<ChipContent>,
    cursor: usize,
    focused: bool,
) -> Line<'static> {
    let mut spans = vec![
        home_draw::chip(label, palette.accent).bold(),
        Span::raw(" "),
    ];
    for (i, content) in chips.into_iter().enumerate() {
        let base = if content.picked {
            palette.accent.bold().reversed()
        } else {
            palette.base
        };
        let style = if focused && i == cursor {
            base.underlined()
        } else {
            base
        };
        spans.push(Span::styled(" ", style));
        for span in content.spans {
            let own = if content.picked {
                span.style.patch(style)
            } else {
                style.patch(span.style)
            };
            spans.push(Span::styled(span.content, own));
        }
        spans.push(Span::styled(" ", style));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

/// An identity as the app's `IdentityPips`: the name unless it is a single color, then the pips in their mana colors.
pub(crate) fn identity_spans(palette: &Palette, identity: &[Color]) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    if let Some(name) = playstyle::identity_label(identity) {
        spans.push(Span::raw(name));
        if !identity.is_empty() {
            spans.push(Span::raw(" "));
        }
    }
    for color in identity {
        let index = Color::all().iter().position(|c| c == color).unwrap_or(0);
        spans.push(mana::pip(palette, index));
    }
    spans
}

/// The Show row: its label, All, then one chip per identity.
pub(crate) fn show_line(
    palette: &Palette,
    identities: &[Vec<Color>],
    picked: Option<&[Color]>,
    cursor: usize,
    focused: bool,
) -> Line<'static> {
    let mut chips = vec![ChipContent::text("All", picked.is_none())];
    chips.extend(identities.iter().map(|identity| ChipContent {
        spans: identity_spans(palette, identity),
        picked: picked == Some(identity.as_slice()),
    }));
    chip_line(palette, "Show", chips, cursor, focused)
}

/// `line` broken into lines at most `width` cells wide, only at the plain one-space gaps between chips, so no chip splits.
pub(crate) fn wrap_at_gaps(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut chip: Vec<Span<'static>> = Vec::new();
    for span in line.spans {
        let gap = span.content == " " && span.style == Style::default();
        chip.push(span);
        if !gap {
            continue;
        }
        let used: usize = current.iter().map(Span::width).sum();
        let adding: usize = chip.iter().map(Span::width).sum();
        if !current.is_empty() && used.saturating_add(adding) > width {
            lines.push(Line::from(mem::take(&mut current)));
        }
        current.append(&mut chip);
    }
    current.append(&mut chip);
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

/// The loaded column and the range of the focused piece.
fn loaded_lines(shared: &Shared, view: &PlaystyleView, width: u16) -> Body {
    let palette = &shared.palette;
    let inner = usize::from(width)
        .saturating_sub(GUTTER.len())
        .saturating_sub(INDENT.len());
    let mut body = Body::default();
    for (i, &section) in view.sections.iter().enumerate() {
        let Some(collapsible) = view.accordion.sections.get(i) else {
            continue;
        };
        let focused = view.focus == Focus::Header(section);
        let mut header = collapsible.header(palette, focused);
        if collapsible.open && section.topic().is_some() {
            header.spans.push(Span::styled("  ?", palette.muted));
        }
        body.item(vec![header], focused);
        if !collapsible.open {
            continue;
        }
        match section {
            Section::ColorMap => color_map_body(&mut body, shared, view, inner),
            Section::Summary => summary_body(&mut body, palette, view, inner),
            Section::Roles => roles_body(&mut body, shared, view, inner),
            Section::Tags => tags_body(&mut body, shared, view, inner),
        }
    }
    if !view.sections.is_empty() {
        body.line(Line::default());
    }
    body.item(
        wrap_at_gaps(
            show_line(
                palette,
                &view.identities(),
                view.identity.as_deref(),
                view.show_cursor,
                view.focus == Focus::Show,
            ),
            usize::from(width).saturating_sub(GUTTER.len()),
        ),
        view.focus == Focus::Show,
    );
    body.line(Line::default());
    list_body(&mut body, shared, view);
    if !view.staples().is_empty() {
        body.line(Line::default());
        staples_body(&mut body, shared, view);
    }
    body
}

/// `line` centered in `width` cells.
fn centered(line: Line<'static>, width: usize) -> Line<'static> {
    let pad = width.saturating_sub(line.width()) / 2;
    let mut spans = vec![Span::raw(" ".repeat(pad))];
    spans.extend(line.spans);
    Line::from(spans).style(line.style)
}

/// The Color map section: the user's tiles at home's seed with the key under them, or the cold start line.
fn color_map_body(body: &mut Body, shared: &Shared, view: &PlaystyleView, width: usize) {
    let palette = &shared.palette;
    let counts = view
        .profile
        .as_ref()
        .and_then(|p| p.color_map.as_ref())
        .map(ColorMapCounts::from)
        .filter(playstyle::color_map_has_data);
    let Some(counts) = counts else {
        body.line(activity_draw::indented(Line::styled(
            COLD_START,
            palette.muted,
        )));
        return;
    };
    let model = ColorMapModel::new(&counts, home::seed(shared));
    let elapsed = view.map_opened.map_or(Duration::MAX, |t| t.elapsed());
    let map_width = u16::try_from(width).unwrap_or(u16::MAX);
    for line in color_map::lines(palette, &model, elapsed, map_width, MAP_ROWS) {
        body.line(activity_draw::indented(centered(line, width)));
    }
    body.line(color_map::rule(palette, width));
    body.line(activity_draw::indented(centered(
        color_map::key(palette, &counts, None),
        width,
    )));
}

/// A Summary row: the label on the left and its value on the right.
fn summary_row(label: &str, value: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let value_width: usize = value.iter().map(Span::width).sum();
    let pad = width
        .saturating_sub(label.chars().count())
        .saturating_sub(value_width)
        .max(1);
    let mut spans = vec![Span::raw(label.to_owned()), Span::raw(" ".repeat(pad))];
    spans.extend(value);
    Line::from(spans)
}

/// The Summary section: Decks, then Top colors and Top color when the profile has them.
fn summary_body(body: &mut Body, palette: &Palette, view: &PlaystyleView, width: usize) {
    let Some(profile) = &view.profile else {
        return;
    };
    let summary = playstyle::profile_summary(profile);
    body.line(activity_draw::indented(summary_row(
        "Decks",
        vec![Span::raw(summary.decks)],
        width,
    )));
    if let Some(value) = summary.color_set {
        let mut spans = identity_spans(palette, &value.pips);
        spans.push(Span::raw(format!(" {}", value.count)));
        body.line(activity_draw::indented(summary_row(
            "Top colors",
            spans,
            width,
        )));
    }
    if let Some(value) = summary.color {
        let mut spans = Vec::new();
        if let Some(color) = value.pips.first() {
            spans.push(Span::raw(format!("{} ", color.to_long_name())));
        }
        spans.extend(identity_spans(palette, &value.pips));
        spans.push(Span::raw(format!(" {}", value.count)));
        body.line(activity_draw::indented(summary_row(
            "Top color",
            spans,
            width,
        )));
    }
}

/// The Card roles chart: a row per role with its deck count, a revealed role's tags under it.
fn roles_body(body: &mut Body, shared: &Shared, view: &PlaystyleView, width: usize) {
    let palette = &shared.palette;
    let bars = view.role_bars();
    let label_width = bars
        .iter()
        .map(|b| b.label.chars().count().saturating_add(2))
        .max()
        .unwrap_or(0)
        .min(width / 2);
    let track = width
        .saturating_sub(label_width)
        .saturating_sub(1)
        .saturating_sub(5)
        .max(1);
    for (i, bar) in bars.iter().enumerate() {
        let selected = view.open_role.as_deref() == Some(bar.key.as_str());
        let mut lines = vec![activity_draw::indented(role_row(
            palette,
            bar,
            selected,
            label_width,
            track,
        ))];
        if selected {
            lines.extend(
                view.role_tags
                    .lines(palette, &shared.caches.oracle_tags)
                    .into_iter()
                    .map(|l| activity_draw::indented(activity_draw::indented(l))),
            );
        }
        body.item(lines, view.focus == Focus::Role(i));
    }
}

/// One role's row: its uppercase label as a chip when it reveals tags, the bar, then its deck count.
fn role_row(
    palette: &Palette,
    bar: &Bar,
    selected: bool,
    label_width: usize,
    track: usize,
) -> Line<'static> {
    let label = bar.label.to_uppercase();
    let style: Style = match (bar.revealable, selected) {
        (true, true) => palette.accent.bold().reversed(),
        (true, false) => palette.accent,
        (false, _) => palette.base,
    };
    charts::row(
        Span::styled(format!(" {label} "), style),
        label_width,
        bar.pct,
        track,
        palette.accent,
        vec![Span::raw(bar.count.to_string()).bold()],
    )
}

/// A number in a tinted stat chip, padded to a table column.
fn stat_cell(value: Option<u32>, style: Style) -> Vec<Span<'static>> {
    let Some(value) = value else {
        return vec![Span::raw(" ".repeat(COLUMN_WIDTH))];
    };
    let text = format!(" {value} ");
    let pad = COLUMN_WIDTH.saturating_sub(text.chars().count());
    vec![
        Span::styled(text, style.reversed()),
        Span::raw(" ".repeat(pad)),
    ]
}

/// The Oracle tags table: its header with each number column's "?", then a row per tag, an open tag's definition under its row.
fn tags_body(body: &mut Body, shared: &Shared, view: &PlaystyleView, width: usize) {
    let palette = &shared.palette;
    let rows = view.tag_rows();
    let tag_width = rows
        .iter()
        .map(|r| r.slug.chars().count().saturating_add(3))
        .max()
        .unwrap_or(0)
        .max(5)
        .min(width / 2);
    let head_focused = view.focus == Focus::TagHead;
    let mut header = vec![Span::styled(
        format!("{:<tag_width$}", "Tag"),
        palette.muted,
    )];
    for column in super::Column::ALL {
        let text = format!("{} ?", column.label());
        let style = if head_focused && column == view.column {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        let pad = COLUMN_WIDTH.saturating_sub(text.chars().count());
        header.push(Span::styled(text, style));
        header.push(Span::raw(" ".repeat(pad)));
    }
    body.item(
        vec![activity_draw::indented(Line::from(header))],
        head_focused,
    );
    for (i, row) in rows.iter().enumerate() {
        let open = view.open_tag.as_deref() == Some(row.slug.as_str());
        let tag_chip = otag::chip(palette, &row.slug, open);
        let pad = tag_width.saturating_sub(tag_chip.width());
        let mut spans = vec![tag_chip, Span::raw(" ".repeat(pad))];
        spans.extend(stat_cell(Some(row.decks), palette.add));
        spans.extend(stat_cell(Some(row.cards), palette.warning));
        spans.extend(stat_cell(row.impact, palette.accent));
        let mut lines = vec![activity_draw::indented(Line::from(spans))];
        if open {
            lines.extend(
                otag::def_lines(palette, &shared.caches.oracle_tags, &row.slug)
                    .into_iter()
                    .map(|l| activity_draw::indented(activity_draw::indented(l))),
            );
        }
        body.item(lines, view.focus == Focus::Tag(i));
    }
}

/// A group's header: its title and its "?".
fn group_header(palette: &Palette, title: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw(title.to_owned()).bold(),
        Span::styled("  ?", palette.muted),
    ])
}

/// The suggestions: the group of rows, or the line saying why there are none.
fn list_body(body: &mut Body, shared: &Shared, view: &PlaystyleView) {
    let palette = &shared.palette;
    if view.entries.is_empty() {
        body.line(Line::styled(NO_SUGGESTIONS, palette.muted));
        return;
    }
    let shown = view.shown();
    if shown.is_empty() {
        let identity = view.identity.clone().unwrap_or_default();
        body.line(Line::styled(
            playstyle::empty_identity_line(&identity),
            palette.muted,
        ));
        return;
    }
    body.line(group_header(palette, LIST_TITLE));
    let rows: Vec<CardRow<'_>> = shown
        .iter()
        .map(|s| suggestion_row(palette, s, view.deck_count, view.why_open))
        .collect();
    let lines = view.rows.lines(palette, &shared.caches.oracle_tags, &rows);
    let focused_row = view.rows.focused().min(rows.len().saturating_sub(1));
    for (i, row) in lines.into_iter().enumerate() {
        body.item(row, view.focus == Focus::List && i == focused_row);
    }
}

/// The Staples panel: the Type and Color rows, then the group of rows or the line saying none match.
fn staples_body(body: &mut Body, shared: &Shared, view: &PlaystyleView) {
    let palette = &shared.palette;
    let mut types = vec![ChipContent::text("All", view.staple_type.is_none())];
    types.extend(
        view.staple_types()
            .into_iter()
            .map(|t| ChipContent::text(&t.to_string(), view.staple_type == Some(t))),
    );
    body.item(
        vec![chip_line(
            palette,
            "Type",
            types,
            view.type_cursor,
            view.focus == Focus::Types,
        )],
        view.focus == Focus::Types,
    );
    let mut colors = vec![ChipContent::text("All", view.staple_color.is_none())];
    colors.extend(view.staple_colors().into_iter().map(|c| ChipContent {
        spans: identity_spans(palette, &c.pips()),
        picked: view.staple_color == Some(c),
    }));
    body.item(
        vec![chip_line(
            palette,
            "Color",
            colors,
            view.color_cursor,
            view.focus == Focus::Colors,
        )],
        view.focus == Focus::Colors,
    );
    body.line(Line::default());
    let staples = view.matching_staples();
    if staples.is_empty() {
        body.line(Line::styled(NO_STAPLES, palette.muted));
        return;
    }
    body.line(group_header(palette, STAPLES_TITLE));
    let rows: Vec<CardRow<'_>> = staples
        .iter()
        .map(|s| staple_row(palette, s, view.deck_count))
        .collect();
    let lines = view
        .staple_rows
        .lines(palette, &shared.caches.oracle_tags, &rows);
    let focused_row = view.staple_rows.focused().min(rows.len().saturating_sub(1));
    for (i, row) in lines.into_iter().enumerate() {
        body.item(row, view.focus == Focus::Staples && i == focused_row);
    }
}

/// A suggestion as its card row: the reasons in its footer under a muted rule, and Create deck, then Maybeboard and Remove when it has an oracle id.
pub(crate) fn suggestion_row<'card>(
    palette: &Palette,
    suggestion: &'card HttpCommanderSuggestion,
    deck_count: u32,
    why_open: Option<Uuid>,
) -> CardRow<'card> {
    let card = &suggestion.card;
    let fold = if why_open == Some(card.scryfall_data.id) {
        Reasons::Unfolded
    } else {
        Reasons::Folded
    };
    let mut footer = vec![Line::styled("─".repeat(RULE_WIDTH), palette.muted)];
    footer.extend(reasons_lines(
        palette,
        &suggestion.reasons,
        deck_count,
        fold,
    ));
    let mut row = CardRow::new(card)
        .with_footer(footer)
        .with_action("c", "Create deck");
    if card.scryfall_data.oracle_id.is_some() {
        row = row
            .with_action("m", "Maybeboard")
            .with_action("r", "Remove");
    }
    row
}

/// A staple as its card row: its share of the user's decks in place of the quantity, and its deck count in the footer.
pub(crate) fn staple_row<'card>(
    palette: &Palette,
    staple: &'card HttpStaple,
    deck_count: u32,
) -> CardRow<'card> {
    CardRow::new(&staple.card)
        .with_qty(staple.decks)
        .with_qty_tags(
            playstyle::staple_percent(staple.decks, deck_count)
                .into_iter()
                .collect(),
        )
        .with_footer(vec![Line::from(home_draw::chip(
            &playstyle::staple_decks(staple.decks, deck_count),
            palette.muted,
        ))])
}

/// A suggestion's reasons: the "Why this commander?" chip (or heading), then, when open, the shared tags grouped by role and one line per color the user builds in. Nothing when there are neither.
pub(crate) fn reasons_lines(
    palette: &Palette,
    reasons: &HttpSuggestionReasons,
    deck_count: u32,
    fold: Reasons,
) -> Vec<Line<'static>> {
    let colors = playstyle::color_lines(&reasons.colors, deck_count);
    if reasons.tags.is_empty() && colors.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![match fold {
        Reasons::Folded => Line::from(vec![
            Span::raw("w ").bold(),
            Span::styled(format!(" {WHY} {FOLDED} "), palette.accent),
        ]),
        Reasons::Unfolded => Line::from(vec![
            Span::raw("w ").bold(),
            Span::styled(format!(" {WHY} {UNFOLDED} "), palette.accent.reversed()),
        ]),
        Reasons::Heading => Line::styled(WHY, palette.muted),
    }];
    if fold == Reasons::Folded {
        return lines;
    }
    if !reasons.tags.is_empty() {
        lines.push(Line::from(format!("• {SHARES_TAGS}")));
        let grouped = playstyle::group_shared_tags(&reasons.tags);
        let mut groups: Vec<(String, &[String])> = grouped
            .roles
            .iter()
            .map(|role| {
                let tags = grouped.by_role.get(role).map_or(&[][..], Vec::as_slice);
                (card_role::role_label(role), tags)
            })
            .collect();
        if !grouped.other.is_empty() {
            groups.push((OTHER_TAGS.to_owned(), &grouped.other));
        }
        for (label, tags) in groups {
            let mut spans = vec![Span::raw("  "), Span::raw(label).bold(), Span::raw(":")];
            for tag in tags {
                spans.push(Span::raw(" "));
                spans.push(otag::chip(palette, tag, /* open */ false));
            }
            lines.push(Line::from(spans));
        }
    }
    lines.extend(
        colors
            .into_iter()
            .map(|line| Line::from(format!("• {line}"))),
    );
    lines
}

/// The loading column in the loaded shape, every piece a ghost: the Color map section open on a blooming ghost map and its key, three collapsed section headers, the Show row and the list's header and six rows.
fn skeleton(palette: &Palette, view: &PlaystyleView, width: u16) -> Vec<Line<'static>> {
    let inner = usize::from(width)
        .saturating_sub(GUTTER.len())
        .saturating_sub(INDENT.len());
    let ghost = |n: usize| Span::styled(GHOST.repeat(n.max(1)), palette.muted);
    let header = |n: usize, open: bool| {
        let mark = if open { "▾ " } else { "▸ " };
        let mut spans = vec![Span::styled(mark, palette.muted), ghost(n)];
        if open {
            spans.push(Span::raw("  "));
            spans.push(ghost(1));
        }
        Line::from(spans)
    };
    let mut body = Body::default();
    body.line(header(9, /* open */ true));
    let map_width = u16::try_from(inner).unwrap_or(u16::MAX);
    for line in color_map::ghost_lines(
        palette,
        &crate::domain::color_map::ghost_model(),
        view.opened.elapsed(),
        map_width,
        MAP_ROWS,
    ) {
        body.line(activity_draw::indented(centered(line, inner)));
    }
    body.line(Line::default());
    let mut key = Vec::new();
    for _ in 0..5 {
        key.push(ghost(3));
        key.push(Span::raw(" "));
    }
    key.push(ghost(9));
    body.line(activity_draw::indented(centered(Line::from(key), inner)));
    body.line(header(7, /* open */ false));
    body.line(header(10, /* open */ false));
    body.line(header(11, /* open */ false));
    body.line(Line::default());
    let mut chips = vec![
        Span::styled(GHOST.repeat(4), palette.accent),
        Span::raw(" "),
    ];
    for size in GHOST_CHIPS {
        chips.push(ghost(size));
        chips.push(Span::raw(" "));
    }
    body.line(Line::from(chips));
    body.line(Line::default());
    body.line(Line::from(vec![ghost(20), Span::raw("  "), ghost(1)]));
    for i in 0..GHOST_ROWS {
        let share = [62, 48, 71, 55].get(i % 4).copied().unwrap_or(60);
        body.line(Line::from(vec![
            ghost(2),
            Span::raw(" "),
            ghost(inner.saturating_mul(share) / 100),
        ]));
    }
    body.lines
}
