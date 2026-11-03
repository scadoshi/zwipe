//! Config: the Appearance and Cards lists with their buttons, the action bar, and the Themes and Exceptions sheets on the sheet widget.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::user::preferences;
use zwipe_ui::screens::config::{
    self, APPEARANCE, CARDS, CHANGE, EXCEPTIONS, PIPS, THEME, UNIVERSES_BEYOND,
};

use super::{ConfigView, Sheet, Stop, ThemeSheet};
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::Shared;
use crate::inbound::widgets::button::{self, Button};
use crate::inbound::widgets::{action_bar, measure, sheet};

/// The lists are at most this wide, in cells.
const WIDTH: u16 = 72;

/// The labels take this many cells.
const LABEL_WIDTH: usize = 22;

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
            ("Esc", "back"),
        ],
    );
    let bar = action_bar::with_help(palette, bar, action_bar::Hint::Question).centered();
    let area = f.area();
    let bar_height = action_bar::height(&bar, area.width);
    let width = WIDTH.min(area.width);
    let lines = lines(palette, view, usize::from(width));
    let [body, keys] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(bar_height)]).areas(area);
    let body = body.centered_horizontally(Constraint::Length(width));
    f.render_widget(Paragraph::new(lines), body);
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    match &view.sheet {
        Some(Sheet::Themes(themes)) => {
            themes_sheet(palette, view, themes).draw(f, palette, f.area());
        }
        Some(Sheet::Exceptions { chip }) => {
            let width = usize::from(f.area().width.saturating_sub(4));
            exceptions_sheet(palette, view, *chip, width).draw(f, palette, f.area());
        }
        None => {}
    }
}

/// A button on a row, highlighted while focused and muted while disabled.
fn button(palette: &Palette, view: &ConfigView, stop: Stop, label: &str) -> Vec<Span<'static>> {
    button::spans(
        palette,
        &Button::new(label).disabled(!view.enabled(stop)),
        view.focused() == stop && view.sheet.is_none(),
    )
}

/// A row's `[?]`, focused while one of `stops` has the focus, since `i` then explains the row.
fn about(palette: &Palette, view: &ConfigView, stops: &[Stop]) -> Vec<Span<'static>> {
    let focused = view.sheet.is_none() && stops.contains(&view.focused());
    button::about(palette, focused)
}

/// A row: its label and `[?]`, its value, its buttons on the right.
fn row(
    width: usize,
    label: &str,
    about: Vec<Span<'static>>,
    value: &str,
    buttons: Vec<Span<'static>>,
) -> Line<'static> {
    let used = label
        .chars()
        .count()
        .saturating_add(1)
        .saturating_add(usize::from(button::ABOUT_WIDTH));
    let mut spans = vec![Span::raw(format!("{label} "))];
    spans.extend(about);
    spans.push(Span::raw(" ".repeat(LABEL_WIDTH.saturating_sub(used))));
    spans.push(Span::raw(value.to_owned()));
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
    let model = &view.model;
    let mut exceptions = row(
        width,
        EXCEPTIONS,
        about(palette, view, &[Stop::Exceptions]),
        "",
        button(palette, view, Stop::Exceptions, CHANGE),
    );
    if model.exceptions_muted() {
        exceptions = exceptions.style(Style::new().dim());
    }
    let mut lines = vec![
        section(APPEARANCE),
        row(
            width,
            THEME,
            about(palette, view, &[Stop::Mode, Stop::Theme]),
            &preferences::display_theme_name(&shown.theme),
            [
                button(palette, view, Stop::Mode, mode),
                vec![Span::raw(" ")],
                button(palette, view, Stop::Theme, CHANGE),
            ]
            .concat(),
        ),
        row(
            width,
            PIPS,
            about(palette, view, &[Stop::Pips]),
            "",
            button(
                palette,
                view,
                Stop::Pips,
                config::pips_label(shown.pip_style),
            ),
        ),
        Line::default(),
        section(CARDS),
        row(
            width,
            UNIVERSES_BEYOND,
            about(palette, view, &[Stop::UniversesBeyond]),
            "",
            button(
                palette,
                view,
                Stop::UniversesBeyond,
                model.universes_beyond_label(),
            ),
        ),
        exceptions,
    ];
    let names = model.exception_names();
    if model.shows_exception_chips() {
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
        .with_footer("Enter", "Save")
        .with_footer("Esc", "Back")
}

/// The Exceptions sheet: the `{n} selected ×` row while the draft has picks, then a chip per franchise, picked ones lit.
fn exceptions_sheet(
    palette: &Palette,
    view: &ConfigView,
    chip: usize,
    width: usize,
) -> sheet::Sheet {
    let draft = view.model.draft();
    let mut lines = Vec::new();
    if !draft.is_empty() {
        lines.push(
            Line::from(vec![
                Span::raw(format!("{}  ", config::selected_line(draft.len()))),
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
            let style = if view.model.is_drafted(&franchise.slug) {
                palette.accent.reversed().bold()
            } else {
                palette.muted
            };
            let style = if i == chip { style.underlined() } else { style };
            Span::styled(format!(" {} ", franchise.name), style)
        })
        .collect();
    let chip_lines = wrap_chips(chips, width);
    let focus = lines.len().saturating_add(focus_line(view, chip, width));
    lines.extend(chip_lines);
    sheet::Sheet::new(EXCEPTIONS, lines)
        .with_hint()
        .with_focus(focus)
        .with_footer("Space", "pick")
        .with_footer("x", "clear")
        .with_footer("Enter", "Save")
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
