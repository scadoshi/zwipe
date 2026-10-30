//! The config screen: appearance and card rows, and the theme and exceptions lists over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap,
};
use zwipe_core::domain::card::scryfall_data::universe::franchise_by_slug;
use zwipe_core::domain::user::preferences::{PipStyle, UserPreferences, display_theme_name};

use super::hints;
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::{App, ConfigRow, ConfigView, ExceptionsEditor, ThemePicker};

/// The labels take this many cells before their values.
const LABEL_WIDTH: usize = 20;

/// A list over the screen is at most this wide and this tall, in cells.
const LIST_WIDTH: u16 = 48;
const LIST_HEIGHT: u16 = 30;

pub(super) fn draw(f: &mut Frame<'_>, palette: &Palette, app: &App, view: &ConfigView) {
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    f.render_widget(
        Line::from(Span::styled("Config", palette.accent).bold()),
        header,
    );
    let lines = view.shown.as_ref().map_or_else(
        || vec![Line::from("Loading...").style(palette.muted)],
        |shown| lines(palette, view, shown),
    );
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(palette.border)),
        body,
    );
    f.render_widget(Line::from(app.status.clone()).style(palette.muted), status);
    f.render_widget(
        hints(
            palette,
            &[
                ("j/k", "move"),
                ("Enter", "change"),
                ("Esc", "home"),
                ("q", "quit"),
            ],
        ),
        keys,
    );

    if let Some(picker) = &view.picker {
        draw_picker(f, palette, picker);
    } else if let Some(editor) = &view.exceptions {
        draw_exceptions(f, palette, editor);
    }
}

fn lines(palette: &Palette, view: &ConfigView, shown: &UserPreferences) -> Vec<Line<'static>> {
    let row = |row: ConfigRow, label: &str, value: String, dim: bool| {
        let marked = view.selected() == row;
        let line = Line::from(vec![
            Span::raw(if marked { "> " } else { "  " }),
            Span::styled(format!("{label:<LABEL_WIDTH$}"), palette.muted),
            Span::raw(value),
        ]);
        let line = if marked { line.bold() } else { line };
        if dim { line.style(palette.muted) } else { line }
    };
    let mode = if shown.dark_mode { "Dark" } else { "Light" };
    let pips = match shown.pip_style {
        PipStyle::Theme => "Theme",
        PipStyle::Natural => "Natural",
    };
    let hidden = shown.exclude_universes_beyond;
    let mut exceptions: Vec<&str> = shown
        .universes_beyond_exceptions
        .iter()
        .filter_map(|slug| franchise_by_slug(slug).map(|f| f.name))
        .collect();
    exceptions.sort_unstable();
    let exceptions = if exceptions.is_empty() {
        "None".to_owned()
    } else {
        exceptions.join(", ")
    };

    vec![
        Line::from(Span::styled("Appearance", palette.accent)),
        row(
            ConfigRow::Theme,
            "Theme",
            display_theme_name(&shown.theme),
            false,
        ),
        row(ConfigRow::Mode, "Mode", mode.to_owned(), false),
        row(ConfigRow::Pips, "Pips", pips.to_owned(), false),
        Line::default(),
        Line::from(Span::styled("Cards", palette.accent)),
        row(
            ConfigRow::UniversesBeyond,
            "Universes Beyond",
            if hidden { "Hide" } else { "Show" }.to_owned(),
            false,
        ),
        row(ConfigRow::Exceptions, "Exceptions", exceptions, !hidden),
    ]
}

fn draw_picker(f: &mut Frame<'_>, palette: &Palette, picker: &ThemePicker) {
    let items: Vec<ListItem<'_>> = theme::picker_rows()
        .into_iter()
        .map(|row| match row {
            PickerRow::Header(label) => {
                ListItem::new(Line::from(label).style(palette.muted).bold())
            }
            PickerRow::Theme(slug) => ListItem::new(display_theme_name(slug)),
        })
        .collect();
    draw_list(
        f,
        palette,
        "Themes",
        items,
        picker.row,
        &[("j/k", "preview"), ("Enter", "save"), ("Esc", "back")],
    );
}

fn draw_exceptions(f: &mut Frame<'_>, palette: &Palette, editor: &ExceptionsEditor) {
    let Some(franchises) = &editor.franchises else {
        draw_list(
            f,
            palette,
            "Exceptions",
            vec![ListItem::new(Line::from("Loading...").style(palette.muted))],
            0,
            &[("Esc", "back")],
        );
        return;
    };
    let items: Vec<ListItem<'_>> = franchises
        .iter()
        .map(|franchise| {
            let picked = editor.draft.contains(&franchise.slug);
            let mark = if picked { "[x] " } else { "[ ] " };
            ListItem::new(Line::from(vec![
                Span::styled(mark, if picked { palette.add } else { palette.muted }),
                Span::raw(franchise.name.clone()),
            ]))
        })
        .collect();
    let title = if editor.draft.is_empty() {
        "Exceptions".to_owned()
    } else {
        format!("Exceptions: {} selected", editor.draft.len())
    };
    draw_list(
        f,
        palette,
        &title,
        items,
        editor.row,
        &[
            ("Space", "pick"),
            ("x", "clear"),
            ("Enter", "save"),
            ("Esc", "back"),
        ],
    );
}

/// A titled, scrolling list centered over the screen, with key hints on its bottom edge.
fn draw_list(
    f: &mut Frame<'_>,
    palette: &Palette,
    title: &str,
    items: Vec<ListItem<'_>>,
    selected: usize,
    keys: &[(&str, &str)],
) {
    let height = u16::try_from(items.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2)
        .min(LIST_HEIGHT);
    let area = f
        .area()
        .centered(Constraint::Max(LIST_WIDTH), Constraint::Length(height));
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(format!(" {title} "))
                .title_bottom(hints(palette, keys))
                .border_style(palette.add)
                .style(palette.base),
        )
        .highlight_style(palette.accent.bold())
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);
    f.render_widget(Clear, area);
    f.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_selected(Some(selected)),
    );
}
