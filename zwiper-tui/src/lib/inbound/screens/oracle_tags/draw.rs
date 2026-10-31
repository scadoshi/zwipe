//! The dictionary: the title, then the search box, the letter rail and the tags, or the skeleton while the catalog loads, or the failure line.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::oracle_tag::OracleTag;

use super::{FAILED, OTHER, OracleTagsView, TITLE, letters};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::home::cache::Catalog;
use crate::inbound::widgets::text_input::BOX_HEIGHT;
use crate::inbound::widgets::{action_bar, header, item_list, skeleton};

/// Ghost letters on the skeleton's rail, as the app draws.
const GHOST_LETTERS: usize = 12;

/// Ghost rows on the skeleton's list, as the app draws.
const GHOST_ROWS: usize = 7;

/// The cells each letter takes on the rail: a space and the letter.
const LETTER_CELLS: u16 = 2;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &OracleTagsView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let hints: &[(&str, &str)] = match &shared.caches.oracle_tags {
        Catalog::Loaded { .. } => {
            if view.typing {
                &[("Enter", "Done"), ("Esc", "Clear"), ("↑/↓", "Move")]
            } else {
                &[
                    ("Esc", "Back"),
                    ("/", "Search"),
                    ("←/→", "Letter"),
                    ("j/k", "Move"),
                    ("x", "Examples"),
                    ("u", "Use"),
                    ("?", "Hint"),
                ]
            }
        }
        Catalog::Failed | Catalog::Idle | Catalog::Loading => &[("Esc", "Back"), ("?", "Hint")],
    };
    let bar = action_bar::line(palette, hints);
    let bar_height = action_bar::height(&bar, area.width);
    let [title, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(bar_height),
    ])
    .areas(area);
    header::draw(f, palette, title, TITLE, header::Corner::Hint);
    match &shared.caches.oracle_tags {
        Catalog::Loaded { tags, .. } => draw_loaded(f, palette, view, tags, body),
        Catalog::Failed => f.render_widget(
            Paragraph::new(Line::styled(FAILED, palette.muted)).centered(),
            body.centered_vertically(Constraint::Length(1)),
        ),
        Catalog::Idle | Catalog::Loading => draw_skeleton(f, palette, body),
    }
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
}

fn draw_loaded(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &OracleTagsView,
    tags: &[OracleTag],
    area: Rect,
) {
    let [search, rail, _, list] = Layout::vertical([
        Constraint::Length(BOX_HEIGHT),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    view.search.draw(f, palette, search, "Search", view.typing);
    if view.typing && search.height >= BOX_HEIGHT {
        f.set_cursor_position(view.search.cursor_position(search));
    }
    let query = view.query();
    draw_rail(f, palette, view, tags, query.is_empty(), rail);

    let rows = view.rows(tags);
    if rows.is_empty() {
        let empty = if query.is_empty() {
            format!("No tags start with {}.", view.letter.to_ascii_uppercase())
        } else {
            format!("No tags match \u{201c}{query}\u{201d}.")
        };
        f.render_widget(Line::styled(empty, palette.muted), list);
        return;
    }
    let selected = view.selected.min(rows.len().saturating_sub(1));
    let rule_width = usize::from(list.width.saturating_sub(2));
    let items: Vec<Vec<Line<'static>>> = rows
        .iter()
        .enumerate()
        .map(|(i, tag)| row(palette, tag, i == selected, rule_width))
        .collect();
    item_list::draw(f, palette, list, &items, selected);
}

/// One tag: its slug, its description, its parents, the keys when selected, and a rule under it.
fn row(
    palette: &Palette,
    tag: &OracleTag,
    selected: bool,
    rule_width: usize,
) -> Vec<Line<'static>> {
    let slug_style = if selected {
        palette.accent.bold()
    } else {
        palette.base.bold()
    };
    let mut lines = vec![Line::styled(tag.slug.clone(), slug_style)];
    lines.push(match &tag.description {
        Some(description) => Line::from(description.clone()),
        None => Line::styled("No description yet", palette.muted),
    });
    if !tag.parent_slugs.is_empty() {
        let mut parents = Vec::new();
        for (i, parent) in tag.parent_slugs.iter().enumerate() {
            if i > 0 {
                parents.push(Span::raw("  "));
            }
            parents.push(Span::styled(parent.clone(), palette.muted));
        }
        lines.push(Line::from(parents));
    }
    if selected {
        lines.push(action_bar::line(
            palette,
            &[("x", "Examples"), ("u", "Use")],
        ));
    }
    lines.push(Line::styled("─".repeat(rule_width), palette.border));
    lines
}

/// The rail of letters, the browsed one picked out while not searching, scrolled so it stays in view.
fn draw_rail(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &OracleTagsView,
    tags: &[OracleTag],
    browsing: bool,
    area: Rect,
) {
    let letters = letters(tags);
    let mut spans = Vec::new();
    for &letter in &letters {
        let label = if letter == OTHER {
            OTHER.to_string()
        } else {
            letter.to_ascii_uppercase().to_string()
        };
        let style = if browsing && letter == view.letter {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(label, style));
    }
    let at = letters.iter().position(|&l| l == view.letter).unwrap_or(0);
    let end = u16::try_from(at.saturating_add(1))
        .unwrap_or(u16::MAX)
        .saturating_mul(LETTER_CELLS);
    let scroll = end.saturating_sub(area.width);
    f.render_widget(Paragraph::new(Line::from(spans)).scroll((0, scroll)), area);
}

/// Ghosts of the search box, the rail and the rows, with no text or controls.
fn draw_skeleton(f: &mut Frame<'_>, palette: &Palette, area: Rect) {
    let width = usize::from(area.width);
    let mut lines = vec![
        Line::default(),
        Line::styled("░".repeat(width), palette.muted),
        Line::default(),
        Line::styled(" ░".repeat(GHOST_LETTERS), palette.muted),
        Line::default(),
    ];
    let slug = "░".repeat((width.saturating_mul(30) / 100).max(1));
    for description in skeleton::rows(palette, GHOST_ROWS, width) {
        lines.push(Line::styled(slug.clone(), palette.muted));
        lines.push(description);
        lines.push(Line::default());
    }
    f.render_widget(Paragraph::new(lines), area);
}
