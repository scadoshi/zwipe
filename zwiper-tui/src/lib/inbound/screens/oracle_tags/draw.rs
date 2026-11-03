//! The dictionary: the title, then the search box, the letter rail and the tags, or the skeleton while the catalog loads, or the failure line.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_ui::screens::oracle_tag_dictionary::{self as model, Body, List};

use super::{OracleTagsView, catalog};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::home::cache::Catalog;
use crate::inbound::widgets::text_input::BOX_HEIGHT;
use crate::inbound::widgets::{action_bar, item_list, skeleton};

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
                ]
            }
        }
        Catalog::Failed | Catalog::Idle | Catalog::Loading => &[("Esc", "Back")],
    };
    let hint = if view.typing && shared.caches.oracle_tags.tags().is_some() {
        action_bar::Hint::Absent
    } else {
        action_bar::Hint::Question
    };
    let bar = action_bar::with_help(palette, action_bar::line(palette, hints), hint);
    let bar_height = action_bar::height(&bar, area.width);
    let [body, keys] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(bar_height)]).areas(area);
    match view.model.body(catalog(&shared.caches.oracle_tags)) {
        Body::Browse { rail, list } => draw_loaded(f, palette, view, &rail, list, body),
        Body::Failed(line) => f.render_widget(
            Paragraph::new(Line::styled(line, palette.muted)).centered(),
            body.centered_vertically(Constraint::Length(1)),
        ),
        Body::Loading => draw_skeleton(f, palette, body),
    }
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
}

fn draw_loaded(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &OracleTagsView,
    rail_letters: &[model::RailLetter],
    list_body: List,
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
    draw_rail(f, palette, view, rail_letters, rail);

    let rows = match list_body {
        List::Rows(rows) => rows,
        List::Empty(line) => {
            f.render_widget(Line::styled(line, palette.muted), list);
            return;
        }
    };
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
        palette.text().bold()
    };
    let mut lines = vec![Line::styled(tag.slug.clone(), slug_style)];
    lines.push(match &tag.description {
        Some(description) => Line::from(description.clone()),
        None => Line::styled(model::NO_DESCRIPTION, palette.muted),
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

/// The rail of letters, the picked one standing out, scrolled so the browsed letter stays in view.
fn draw_rail(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &OracleTagsView,
    letters: &[model::RailLetter],
    area: Rect,
) {
    let mut spans = Vec::new();
    for chip in letters {
        let style = if chip.picked {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(chip.label.clone(), style));
    }
    let at = letters
        .iter()
        .position(|chip| chip.letter == view.model.letter())
        .unwrap_or(0);
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
