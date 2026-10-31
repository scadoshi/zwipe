//! Config: the header, the Appearance and Cards lists with their buttons, the action bar, and the Themes and Exceptions sheets on the sheet widget.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::scryfall_data::universe;
use zwipe_core::domain::user::preferences;
use zwipe_core::domain::user::preferences::PipStyle;

use super::{ConfigView, ExceptionsSheet, Sheet, Stop, ThemeSheet, same_set};
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar::{self, Button};
use crate::inbound::widgets::header::{self, Corner};
use crate::inbound::widgets::{measure, sheet};

/// The lists are at most this wide, in cells.
const WIDTH: u16 = 72;

/// The labels take this many cells.
const LABEL_WIDTH: usize = 20;

/// One swatch dot.
const SWATCH: &str = "●";

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ConfigView) {
    let palette = &shared.palette;
    let bar = action_bar::line(
        palette,
        &[
            ("j/k", "move"),
            ("Enter", "select"),
            ("i", "about row"),
            ("?", "hint"),
            ("!", "help"),
            ("Esc", "back"),
        ],
    )
    .centered();
    let bar_height = action_bar::height(&bar, f.area().width);
    let [head, _, body, buttons, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(bar_height),
    ])
    .areas(f.area());
    header::draw(f, palette, head, "Config", Corner::Hint);
    let body = body.centered_horizontally(Constraint::Length(WIDTH.min(body.width)));
    f.render_widget(
        Paragraph::new(lines(palette, view, usize::from(body.width))),
        body,
    );
    f.render_widget(
        action_bar::buttons(palette, &[Button::new("Back")], None).centered(),
        buttons,
    );
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    match &view.sheet {
        Some(Sheet::Themes(themes)) => {
            themes_sheet(palette, view, themes).draw(f, palette, f.area());
        }
        Some(Sheet::Exceptions(exceptions)) => {
            let width = usize::from(f.area().width.saturating_sub(4));
            exceptions_sheet(palette, view, exceptions, width).draw(f, palette, f.area());
        }
        None => {}
    }
}

/// A button on a row, reversed while focused and muted while disabled.
fn button(palette: &Palette, view: &ConfigView, stop: Stop, label: &str) -> Span<'static> {
    let style: Style = if view.enabled(stop) {
        palette.accent
    } else {
        palette.muted
    };
    let style = if view.focused() == stop && view.sheet.is_none() {
        style.bold().reversed()
    } else {
        style
    };
    Span::styled(format!(" {label} "), style)
}

/// A row: its label and `?`, its value, its buttons on the right.
fn row(
    palette: &Palette,
    width: usize,
    label: &str,
    value: &str,
    buttons: Vec<Span<'static>>,
) -> Line<'static> {
    let question = palette.swatches.get(5).copied().unwrap_or(palette.muted);
    let mut spans = vec![
        Span::raw(label.to_owned()),
        Span::styled(" ?", question),
        Span::raw(" ".repeat(LABEL_WIDTH.saturating_sub(label.chars().count().saturating_add(2)))),
        Span::raw(value.to_owned()),
    ];
    let used: usize = spans.iter().map(Span::width).sum();
    let tail: usize = buttons.iter().map(Span::width).sum();
    spans.push(Span::raw(
        " ".repeat(width.saturating_sub(used).saturating_sub(tail)),
    ));
    spans.extend(buttons);
    Line::from(spans)
}

fn lines(palette: &Palette, view: &ConfigView, width: usize) -> Vec<Line<'static>> {
    let shown = &view.shown;
    let section = |title: &str| Line::from(title.to_owned()).style(palette.muted).bold();
    let mode = if shown.dark_mode {
        "☾ Dark"
    } else {
        "☀ Light"
    };
    let pips = match shown.pip_style {
        PipStyle::Theme => "Theme",
        PipStyle::Natural => "Natural",
    };
    let hidden = shown.exclude_universes_beyond;
    let mut exceptions = row(
        palette,
        width,
        "Exceptions",
        "",
        vec![button(palette, view, Stop::Exceptions, "Change")],
    );
    if !hidden {
        exceptions = exceptions.style(Style::new().dim());
    }
    let mut lines = vec![
        section("Appearance"),
        row(
            palette,
            width,
            "Theme",
            &preferences::display_theme_name(&shown.theme),
            vec![
                button(palette, view, Stop::Mode, mode),
                Span::raw(" "),
                button(palette, view, Stop::Theme, "Change"),
            ],
        ),
        row(
            palette,
            width,
            "Pips",
            "",
            vec![button(palette, view, Stop::Pips, pips)],
        ),
        Line::default(),
        section("Cards"),
        row(
            palette,
            width,
            "Universes Beyond",
            "",
            vec![button(
                palette,
                view,
                Stop::UniversesBeyond,
                if hidden { "Hide" } else { "Show" },
            )],
        ),
        exceptions,
    ];
    let mut names: Vec<&str> = shown
        .universes_beyond_exceptions
        .iter()
        .filter_map(|slug| universe::franchise_by_slug(slug).map(|f| f.name))
        .collect();
    names.sort_unstable();
    if !names.is_empty() {
        lines.extend(wrap_chips(
            names
                .iter()
                .map(|n| Span::styled(format!(" {n} "), palette.muted.reversed()))
                .collect(),
            width,
        ));
    }
    lines
}

/// Lays `chips` out in lines of at most `width` cells, a space between chips.
fn wrap_chips(chips: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0_usize;
    for chip in chips {
        let w = chip.width().saturating_add(1);
        if used > 0 && used.saturating_add(w) > width {
            lines.push(Line::from(std::mem::take(&mut current)).centered());
            used = 0;
        }
        used = used.saturating_add(w);
        current.push(chip);
        current.push(Span::raw(" "));
    }
    if !current.is_empty() {
        lines.push(Line::from(current).centered());
    }
    lines
}

/// The Themes sheet: each section label, then a row per theme with its six swatches in the current mode, the selection highlighted.
fn themes_sheet(palette: &Palette, view: &ConfigView, themes: &ThemeSheet) -> sheet::Sheet {
    let dark = view.shown.dark_mode;
    let lines: Vec<Line<'static>> = theme::picker_rows()
        .into_iter()
        .enumerate()
        .map(|(i, row)| match row {
            PickerRow::Header(label) => Line::from(label).style(palette.tertiary).bold(),
            PickerRow::Theme(slug) => {
                let name = preferences::display_theme_name(slug);
                let mut spans = vec![Span::raw(format!(
                    "  {name}{}",
                    " ".repeat(28_usize.saturating_sub(usize::from(measure::width_of(&name))))
                ))];
                spans.extend(
                    theme::Palette::swatches_of(slug, dark)
                        .into_iter()
                        .map(|style| Span::styled(SWATCH, style)),
                );
                let line = Line::from(spans);
                if i == themes.row {
                    line.style(palette.accent.bold().reversed())
                } else {
                    line
                }
            }
        })
        .collect();
    sheet::Sheet::new("Themes", lines)
        .with_focus(themes.row)
        .with_footer("j/k", "preview")
        .with_footer("Enter", if themes.picked() { "Save" } else { "-" })
        .with_footer("Esc", "Back")
}

/// The Exceptions sheet: the `{n} selected ×` row while the draft has picks, then a chip per franchise, picked ones lit.
fn exceptions_sheet(
    palette: &Palette,
    view: &ConfigView,
    exceptions: &ExceptionsSheet,
    width: usize,
) -> sheet::Sheet {
    let mut lines = Vec::new();
    if !exceptions.draft.is_empty() {
        lines.push(
            Line::from(vec![
                Span::raw(format!("{} selected  ", exceptions.draft.len())),
                Span::styled("\u{d7}", palette.error).bold(),
            ])
            .centered(),
        );
    }
    let chips: Vec<Span<'static>> = view
        .franchises
        .iter()
        .enumerate()
        .map(|(i, franchise)| {
            let style = if exceptions.draft.contains(&franchise.slug) {
                palette.accent.reversed().bold()
            } else {
                palette.muted
            };
            let style = if i == exceptions.chip {
                style.underlined()
            } else {
                style
            };
            Span::styled(format!(" {} ", franchise.name), style)
        })
        .collect();
    let chip_lines = wrap_chips(chips, width);
    let focus = lines
        .len()
        .saturating_add(focus_line(view, exceptions.chip, width));
    lines.extend(chip_lines);
    let save = if same_set(&exceptions.draft, &view.shown.universes_beyond_exceptions) {
        "-"
    } else {
        "Save"
    };
    sheet::Sheet::new("Exceptions", lines)
        .with_hint()
        .with_focus(focus)
        .with_footer("Space", "pick")
        .with_footer("x", "clear")
        .with_footer("Enter", save)
        .with_footer("Esc", "Back")
}

/// The chip line the chip at `chip` falls on.
fn focus_line(view: &ConfigView, chip: usize, width: usize) -> usize {
    let mut line = 0_usize;
    let mut used = 0_usize;
    for franchise in view.franchises.iter().take(chip.saturating_add(1)) {
        let w = franchise.name.chars().count().saturating_add(3);
        if used > 0 && used.saturating_add(w) > width {
            line = line.saturating_add(1);
            used = 0;
        }
        used = used.saturating_add(w);
    }
    line
}
