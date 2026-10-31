//! The Commander maybeboard: the title, the Quick add box, then one scrolled column of the result chips, the Show row and the saved rows, then the action bar.
//!
//! The app hangs the result chips over the content below the box; here they take their own lines between the box and the Show row, so the Show row stays in view while the focus is on it. The box's title stands in for the app's "Quick add" label chip.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;

use super::{
    Focus, LIST_TITLE, LOAD_FAILED, MaybeboardView, NO_COMMANDERS, NO_MATCH, NO_RESULTS, Overlay,
    QUICK_ADD, SEARCHING,
};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::activity::draw::{Body, GHOST, GUTTER, draw_lines};
use crate::inbound::screens::card_sheets::filter::draw as filter_sheet;
use crate::inbound::screens::playstyle::draw::{ChipContent, chip_line, wrap_at_gaps};
use crate::inbound::widgets::card_rows::CardRow;
use crate::inbound::widgets::text_input::BOX_HEIGHT;
use crate::inbound::widgets::{action_bar, mana, skeleton};

/// Ghost rows in the list while loading, as the app's skeleton.
const GHOST_ROWS: usize = 6;

/// The Show row's chips while loading, as the app's seven small ghosts.
const GHOST_CHIPS: usize = 7;

/// The app's "×", beside the box while it holds a query.
const CLEAR_MARK: &str = "×";

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &MaybeboardView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let [title, input, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(BOX_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(
        Line::styled(super::TITLE, palette.accent).bold().centered(),
        title,
    );
    if view.is_loading {
        f.render_widget(
            Block::bordered()
                .title(Span::styled(GHOST.repeat(QUICK_ADD.len()), palette.accent))
                .border_style(palette.muted),
            input,
        );
        draw_lines(f, skeleton_lines(palette, body.width), (0, 0), body);
    } else {
        draw_input(f, palette, view, input);
        let lines = loaded_lines(shared, view, body.width);
        draw_lines(f, lines.lines, lines.focus, body);
    }
    f.render_widget(action_bar::line(palette, &key_hints(view)), keys);
    match &view.overlay {
        Some(Overlay::More(sheet)) => sheet.draw(f, palette, f.area()),
        Some(Overlay::Clear(dialog)) => {
            super::keys::more_sheet(palette).draw(f, palette, f.area());
            dialog.draw(f, palette);
        }
        None => {}
    }
    if let Some(sheet) = &view.sheet {
        filter_sheet::draw(f, shared, sheet, f.area());
    }
}

/// The Quick add box, the "×" beside it while it holds a query, and the cursor while typing.
fn draw_input(f: &mut Frame<'_>, palette: &Palette, view: &MaybeboardView, area: Rect) {
    let [field, mark] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)]).areas(area);
    view.quick
        .query
        .draw(f, palette, field, QUICK_ADD, view.focus == Focus::Input);
    if !view.quick.query.is_empty() && mark.height > 1 {
        let row = Rect {
            y: mark.y.saturating_add(1),
            height: 1,
            ..mark
        };
        f.render_widget(Line::styled(format!(" {CLEAR_MARK}"), palette.muted), row);
    }
    if view.quick.typing && field.height >= BOX_HEIGHT && field.width > 2 {
        f.set_cursor_position(view.quick.query.cursor_position(field));
    }
}

/// The keys for what has the focus, then the screen's own: the app's Back, Filter, Swipe and More.
fn key_hints(view: &MaybeboardView) -> Vec<(&'static str, &'static str)> {
    let mut keys = Vec::new();
    if view.quick.typing {
        keys.push(("Enter", "Done"));
        keys.push(("Esc", "Clear"));
        return keys;
    }
    if !view.is_loading {
        keys.push(("j/k", "Move"));
        match view.focus {
            Focus::Input => {
                keys.push(("Enter", "Type"));
                if !view.quick.query.is_empty() {
                    keys.push(("x", "Clear"));
                }
            }
            Focus::Results => {
                keys.push(("h/l", "Chip"));
                keys.push(("Enter", "Add"));
            }
            Focus::Show => {
                keys.push(("h/l", "Chip"));
                keys.push(("Enter", "Pick"));
            }
            Focus::List => keys.push(("Enter", "Expand")),
        }
        keys.push(("/", QUICK_ADD));
    }
    keys.push((
        "f",
        if view.filter_dot() {
            "Filter •"
        } else {
            "Filter"
        },
    ));
    keys.push(("s", "Swipe"));
    keys.push(("M", "More"));
    keys.push(("H", "Hint"));
    keys.push(("Esc", "Back"));
    keys
}

/// A saved commander as its card row: Create deck, then Remove when it has an oracle id.
pub(crate) fn entry_row(card: &Card) -> CardRow<'_> {
    let row = CardRow::new(card).with_action("c", "Create deck");
    if card.scryfall_data.oracle_id.is_some() {
        row.with_action("r", "Remove")
    } else {
        row
    }
}

/// The result chips: one muted chip while searching or when nothing is left to add, else one per commander, the one under the cursor picked out while focused.
pub(crate) fn results_line(palette: &Palette, view: &MaybeboardView) -> Line<'static> {
    let note = |text: &str| Line::from(Span::styled(format!(" {text} "), palette.muted.reversed()));
    if view.quick.searching {
        return note(SEARCHING);
    }
    let shown = view.shown_results();
    if shown.is_empty() {
        return note(NO_RESULTS);
    }
    let focused = view.focus == Focus::Results;
    let mut spans = Vec::new();
    for (i, card) in shown.iter().enumerate() {
        let style = if focused && i == view.quick.cursor {
            palette.accent.bold().reversed()
        } else {
            palette.base.reversed()
        };
        spans.push(Span::styled(
            format!(" {} ", card.scryfall_data.name),
            style,
        ));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

/// The Show row: its label, All, then the five color pips.
pub(crate) fn show_line(palette: &Palette, view: &MaybeboardView) -> Line<'static> {
    let mut chips = vec![ChipContent::text("All", view.colors.is_empty())];
    chips.extend(
        Color::all()
            .iter()
            .enumerate()
            .map(|(index, color)| ChipContent {
                spans: vec![mana::pip(palette, index)],
                picked: view.colors.contains(color),
            }),
    );
    chip_line(
        palette,
        "Show",
        chips,
        view.show_cursor,
        view.focus == Focus::Show,
    )
}

/// The loaded column and the range of the focused piece.
fn loaded_lines(shared: &Shared, view: &MaybeboardView, width: u16) -> Body {
    let palette = &shared.palette;
    let wrap = usize::from(width).saturating_sub(GUTTER.len());
    let mut body = Body::default();
    if view.searching_catalog() {
        body.item(
            wrap_at_gaps(results_line(palette, view), wrap),
            view.focus == Focus::Results,
        );
        body.line(Line::default());
    }
    body.item(
        wrap_at_gaps(show_line(palette, view), wrap),
        view.focus == Focus::Show,
    );
    body.line(Line::default());
    let note = if view.load_failed {
        Some(LOAD_FAILED)
    } else if view.entries.is_empty() {
        Some(NO_COMMANDERS)
    } else if view.filtered().is_empty() {
        Some(NO_MATCH)
    } else {
        None
    };
    if let Some(note) = note {
        body.line(Line::styled(note, palette.muted));
        return body;
    }
    body.line(Line::from(Span::raw(LIST_TITLE).bold()));
    let filtered = view.filtered();
    let rows: Vec<CardRow<'_>> = filtered.iter().map(|&c| entry_row(c)).collect();
    let lines = view.rows.lines(palette, &shared.caches.oracle_tags, &rows);
    let focused_row = view.rows.focused().min(rows.len().saturating_sub(1));
    for (i, row) in lines.into_iter().enumerate() {
        body.item(row, view.focus == Focus::List && i == focused_row);
    }
    body
}

/// The loading column, every piece a ghost: the Show row's label and seven chips, then the list's header and six rows.
fn skeleton_lines(palette: &Palette, width: u16) -> Vec<Line<'static>> {
    let inner = usize::from(width).saturating_sub(GUTTER.len());
    let ghost = |n: usize| Span::styled(GHOST.repeat(n), palette.muted);
    let mut chips = vec![
        Span::styled(GHOST.repeat(4), palette.accent),
        Span::raw(" "),
    ];
    for _ in 0..GHOST_CHIPS {
        chips.push(ghost(3));
        chips.push(Span::raw(" "));
    }
    let mut body = Body::default();
    for line in wrap_at_gaps(Line::from(chips), inner) {
        body.line(line);
    }
    body.line(Line::default());
    body.line(Line::from(ghost(10)));
    for row in skeleton::rows(palette, GHOST_ROWS, inner) {
        body.line(row);
    }
    body.lines
}
