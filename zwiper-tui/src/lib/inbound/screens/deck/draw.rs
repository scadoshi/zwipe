//! One deck: the Profile card and the sections in as many panels as fit, laid out by [`layout`], with the More sheet and dialogs over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Format, deck_tag, draw_odds};
use zwipe_ui::screens::deck_view::{
    self, AVERAGE_CARD_PRICE, AVERAGE_MANA_VALUE, AVERAGE_PT, CANCEL, CLONE_BODY, CLONE_BUSY,
    CLONE_SAVE, LANDS, LOAD_FAILED, Section, Status, TOTAL_PRICE,
};

use super::layout::{self, Pane, Panel};
use super::{CloneDialog, DeckView, Dialog, Item};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::charts::Column;
use crate::inbound::widgets::measure::BORDERS;
use crate::inbound::widgets::pill::{self, PillState};
use crate::inbound::widgets::{action_bar, button, charts, mana, measure, panel, skeleton};

/// A vertical chart is at most this tall, frame included, so a small deck's bars stay short.
const CHART_HEIGHT: u16 = 14;

/// The Roles chart keeps at least this many rows under the Types chart, which gives way first.
const ROLES_MIN_HEIGHT: u16 = 5;

/// The Mana charts sit side by side from this many cells; narrower, they stack.
const MANA_SIDE_BY_SIDE: u16 = 48;

/// A panel's frame and its one-cell margin each side, in cells.
const PANEL_INSET: u16 = BORDERS.saturating_add(2);

/// The profile's labels take this many cells before their values.
const LABEL_WIDTH: usize = 16;

/// A stat's label takes this many cells before its value: the longest, "Average card price", and two spaces.
const STAT_WIDTH: usize = 20;

/// Cards drawn before the first turn's draw: the opening hand.
const OPENING_HAND: u32 = 7;

/// The budget's currencies, in the app's order.
pub(crate) const CURRENCIES: [PriceCurrency; 3] =
    [PriceCurrency::Usd, PriceCurrency::Eur, PriceCurrency::Tix];

/// The color chart's labels, by WUBRG place, for coloring each column.
const COLOR_LABELS: [&str; 5] = ["white", "blue", "black", "red", "green"];

/// Ghost rows the Profile card shows while its profile loads.
const PROFILE_GHOST_ROWS: usize = 4;

/// The ghost standing in for the loading pane's tab title.
const GHOST_TAB: &str = "░░░░░░";

/// The Clone dialog is at most this wide inside its panel, in cells.
const DIALOG_WIDTH: u16 = 56;

/// A key and what it does, for the hints.
type Hint = (&'static str, &'static str);

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &DeckView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let (pane_keys, screen_keys) = key_hints(view);
    let pane_line = action_bar::line(palette, &pane_keys);
    let screen_line = action_bar::with_help(
        palette,
        action_bar::line(palette, &screen_keys),
        action_bar::Hint::Question,
    );
    // The hints take one row when they fit, else the pane's keys go over the screen's, each wrapping on its own.
    let one_row = pane_line.width().saturating_add(screen_line.width()) <= usize::from(area.width);
    let keys_text = if one_row {
        let mut spans = pane_line.spans;
        spans.extend(screen_line.spans);
        action_bar::wrap(Line::from(spans), area.width)
    } else {
        let mut text = action_bar::wrap(pane_line, area.width);
        text.lines
            .extend(action_bar::wrap(screen_line, area.width).lines);
        text
    };
    let keys_height = u16::try_from(keys_text.lines.len()).unwrap_or(u16::MAX);
    let [body, status, keys] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(keys_height),
    ])
    .areas(area);

    let panes = view.panes();
    let profile_rows = profile_lines(
        palette,
        view,
        layout::PROFILE_WIDTH.saturating_sub(PANEL_INSET),
    )
    .len();
    let plan = layout::plan(
        body,
        &panes,
        view.zoomed.then(|| view.focused()),
        measure::height_of(profile_rows).saturating_add(BORDERS),
    );
    for panel in &plan.panels {
        draw_panel(f, palette, view, panel);
    }
    if let Some(note) = plan.note
        && view.model.status() == Status::Loaded
    {
        f.render_widget(
            Line::from("Stats fill in as the deck grows.").style(palette.muted),
            note.inner(Margin::new(1, 0)),
        );
    }

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    f.render_widget(keys_text, keys);

    if let Some(menu) = &view.more {
        menu.draw(f, palette, f.area());
    }
    match &view.dialog {
        Some(
            Dialog::Buy(alert)
            | Dialog::Share(alert)
            | Dialog::ClearSkips(alert)
            | Dialog::Delete(alert),
        ) => alert.draw(f, palette),
        Some(Dialog::Clone(clone)) => draw_clone(f, palette, clone),
        Some(Dialog::Info(popup)) => popup.draw(f, palette),
        None => {}
    }
}

/// The key hints: the focused pane's own keys, then the screen's.
fn key_hints(view: &DeckView) -> (Vec<Hint>, Vec<Hint>) {
    let focused = view.focused();
    let mut pane_keys = vec![("Tab", "panel")];
    if !view.items().is_empty() {
        let press = if focused == Pane::Section(Section::Warnings) {
            "fix"
        } else {
            "open"
        };
        pane_keys.extend([("j/k", "move"), ("Space", press)]);
    } else if view.scroll_limit() > 0 {
        pane_keys.push(("j/k", "scroll"));
    }
    if !view.zoomed {
        pane_keys.push(("Enter", "zoom"));
    }
    match focused {
        Pane::Section(Section::Budget) if view.model.currency_chips_shown() => {
            pane_keys.push(("h/l", "currency"));
        }
        Pane::Section(Section::DrawOdds) => pane_keys.push(("-/+", "turn")),
        _ => {}
    }
    if matches!(focused, Pane::Section(_)) {
        pane_keys.push(("i", "info"));
    }
    let mut screen_keys = vec![("e", "Edit"), ("c", "Cards"), ("m", "More")];
    if view.model.deck().is_some() {
        screen_keys.push(("s", "swipe"));
    }
    screen_keys.extend([
        ("Esc", if view.zoomed { "unzoom" } else { "Back" }),
        ("q", "quit"),
    ]);
    (pane_keys, screen_keys)
}

/// A framed panel showing one of its panes, with its tabs across the top when it holds more than one. The focused panel's frame is thick and in the accent color; Warnings keeps the warning color.
fn draw_panel(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, panel: &Panel) {
    let Some(shown) = view.shown(&panel.panes) else {
        return;
    };
    let has_focus = panel.panes.contains(&view.focused());
    let border = if shown == Pane::Section(Section::Warnings) {
        palette.warning
    } else if has_focus {
        palette.accent
    } else {
        palette.border
    };
    let room = usize::from(panel.area.width.saturating_sub(BORDERS));
    let block = crate::inbound::widgets::panel::bordered(palette)
        .border_style(border)
        .title(tabs(palette, view, &panel.panes, shown, has_focus, room));
    let inner = block.inner(panel.area).inner(Margin::new(1, 0));
    f.render_widget(block, panel.area);
    match shown {
        Pane::Profile => draw_profile(f, palette, view, inner),
        Pane::Loading => f.render_widget(
            Paragraph::new(skeleton::rows(
                palette,
                usize::from(inner.height),
                usize::from(inner.width),
            )),
            inner,
        ),
        Pane::Section(section) => draw_body(f, palette, view, section, inner),
    }
}

/// A pane's title and badge, as its tab or its panel's title reads.
fn tab_label(view: &DeckView, pane: Pane) -> String {
    match pane {
        Pane::Profile => "Profile".to_owned(),
        Pane::Loading => GHOST_TAB.to_owned(),
        Pane::Section(section) => match view.model.badge(section) {
            Some(badge) => format!("{} {badge}", section.title()),
            None => section.title().to_owned(),
        },
    }
}

/// The tab strip for a panel's title, or its one title. The shown tab takes the label style of every box title, and is a bold chip while its panel has the focus; a shown section takes its `[?]` a space after it, lit while its panel has the focus, since `i` opens it then. When the tabs outgrow `room`, the strip keeps the shown tab and as many neighbors as fit, with ‹ and › where tabs are cut.
fn tabs(
    palette: &Palette,
    view: &DeckView,
    panes: &[Pane],
    shown: Pane,
    has_focus: bool,
    room: usize,
) -> Line<'static> {
    let labels: Vec<String> = panes.iter().map(|p| tab_label(view, *p)).collect();
    let at = panes.iter().position(|p| *p == shown).unwrap_or(0);
    let about = matches!(shown, Pane::Section(_));
    let widths: Vec<usize> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let width = usize::from(measure::width_of(l)).saturating_add(2);
            if about && i == at {
                width.saturating_add(usize::from(button::ABOUT_WIDTH))
            } else {
                width
            }
        })
        .collect();
    let (start, end) = tab_window(&widths, at, room);
    let mut spans = Vec::new();
    if start > 0 {
        spans.push(Span::styled("‹", palette.muted));
    }
    for (i, (pane, label)) in panes.iter().zip(&labels).enumerate().take(end).skip(start) {
        let picked = i == at && has_focus && panes.len() > 1;
        // The shown tab reads as the panel's title, a chip only while its panel has the focus.
        let style = if i != at {
            palette.muted
        } else if !picked {
            panel::label_style(palette)
        } else if *pane == Pane::Section(Section::Warnings) {
            palette.warning.bold()
        } else {
            palette.accent.bold()
        };
        spans.push(pill::pill(
            label,
            style,
            PillState {
                picked,
                focused: false,
            },
        ));
        // The chip's own padding is the space between the label and its marker.
        if about && i == at {
            spans.extend(button::about(palette, has_focus));
        }
    }
    if end < panes.len() {
        spans.push(Span::styled("›", palette.muted));
    }
    Line::from(spans)
}

/// The tabs `start..end` that fit in `room` around tab `at`, each as wide as `widths` says. All of them when they fit; else two cells go to the ‹ and › marks and the window grows from `at`, a tab right then a tab left.
fn tab_window(widths: &[usize], at: usize, room: usize) -> (usize, usize) {
    let total = widths.iter().fold(0_usize, |sum, w| sum.saturating_add(*w));
    if total <= room {
        return (0, widths.len());
    }
    let room = room.saturating_sub(2);
    let mut start = at.min(widths.len());
    let mut end = at.saturating_add(1).min(widths.len());
    let mut used = widths.get(at).copied().unwrap_or(0);
    loop {
        let mut grew = false;
        if let Some(w) = widths.get(end)
            && used.saturating_add(*w) <= room
        {
            used = used.saturating_add(*w);
            end = end.saturating_add(1);
            grew = true;
        }
        if let Some(before) = start.checked_sub(1)
            && let Some(w) = widths.get(before)
            && used.saturating_add(*w) <= room
        {
            used = used.saturating_add(*w);
            start = before;
            grew = true;
        }
        if !grew {
            return (start, end);
        }
    }
}

/// The profile's rows for text `width` cells wide; a value that does not fit beside its label goes under it.
fn profile_lines(palette: &Palette, view: &DeckView, width: u16) -> Vec<Line<'static>> {
    let Some(profile) = view.model.profile() else {
        return match view.model.status() {
            Status::Failed => vec![Line::from(LOAD_FAILED).style(palette.muted)],
            Status::Loading | Status::Loaded => {
                skeleton::rows(palette, PROFILE_GHOST_ROWS, usize::from(width))
            }
        };
    };
    let value_width = usize::from(width).saturating_sub(LABEL_WIDTH);
    let mut lines = Vec::new();
    let mut field = |label: &str, value: String| {
        if value.chars().count() <= value_width {
            lines.push(Line::from(vec![
                Span::styled(format!("{label:<LABEL_WIDTH$}"), palette.muted),
                Span::raw(value),
            ]));
        } else {
            lines.push(Line::from(Span::styled(label.to_owned(), palette.muted)));
            lines.push(Line::from(format!("  {value}")));
        }
    };
    field("Name", profile.name.to_string());
    field("Cards", view.model.card_count().to_string());
    field(
        "Format",
        profile
            .format
            .map_or_else(|| "None".to_owned(), |f| f.display_name().to_owned()),
    );
    if let Some(format) = profile.format.filter(Format::has_commander) {
        let label = if format.has_signature_spell() {
            "Oathbreaker"
        } else {
            "Commander"
        };
        field(
            label,
            profile
                .commander_name
                .clone()
                .unwrap_or_else(|| "None".to_owned()),
        );
    }
    for (label, name) in [
        ("Partner", &profile.partner_commander_name),
        ("Background", &profile.background_name),
        ("Signature spell", &profile.signature_spell_name),
    ] {
        if let Some(name) = name {
            field(label, name.clone());
        }
    }
    if let Some(power) = profile.power_level {
        field("Power level", power.display_name().to_owned());
    }
    lines
}

fn draw_profile(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    f.render_widget(
        scrolled(
            Paragraph::new(profile_lines(palette, view, area.width)).wrap(Wrap { trim: true }),
            view.scroll_of(Pane::Profile),
            area,
        ),
        area,
    );
}

fn draw_body(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, section: Section, area: Rect) {
    match section {
        Section::Budget => draw_budget(f, palette, view, area),
        Section::Tags => draw_tags(f, palette, view, area),
        Section::Distributions => {
            if let Some(m) = view.model.charted() {
                draw_distributions(f, palette, view, m, area);
            }
        }
        Section::Mana => draw_mana(f, palette, view, area),
        Section::DrawOdds => {
            if let Some(m) = view.model.charted() {
                draw_draw_odds(
                    f,
                    palette,
                    m,
                    view.turn,
                    view.scroll_of(Pane::Section(Section::DrawOdds)),
                    area,
                );
            }
        }
        Section::Warnings => draw_warnings(f, palette, view, area),
    }
}

/// A stat's label and value on one row, or the value under its label when the row is narrower than `width`.
fn stat(palette: &Palette, label: &str, value: String, width: u16) -> Vec<Line<'static>> {
    if STAT_WIDTH.saturating_add(value.chars().count()) <= usize::from(width) {
        vec![Line::from(vec![
            Span::styled(format!("{label:<STAT_WIDTH$}"), palette.muted),
            Span::raw(value).bold(),
        ])]
    } else {
        vec![
            Line::from(Span::styled(label.to_owned(), palette.muted)),
            Line::from(vec![Span::raw("  "), Span::raw(value).bold()]),
        ]
    }
}

/// The currency chips once there are cards to price, then the total and the average.
fn draw_budget(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let currency = view.model.currency();
    let mut lines = Vec::new();
    if view.model.currency_chips_shown() {
        lines.push(pill::row(
            CURRENCIES
                .iter()
                .map(|c| {
                    pill::pill(
                        c.label(),
                        palette.muted,
                        PillState {
                            picked: *c == currency,
                            focused: false,
                        },
                    )
                })
                .collect(),
        ));
    }
    if let Some(total) = view.model.total_price_line() {
        lines.extend(stat(palette, TOTAL_PRICE, total, area.width));
    }
    if let Some(average) = view.model.average_price_line() {
        lines.extend(stat(palette, AVERAGE_CARD_PRICE, average, area.width));
    }
    f.render_widget(
        scrolled(
            Paragraph::new(lines),
            view.scroll_of(Pane::Section(Section::Budget)),
            area,
        ),
        area,
    );
}

fn draw_tags(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let Some(profile) = view.model.profile() else {
        return;
    };
    let mut lines = Vec::new();
    let mut group = |label: &str, tags: Vec<String>, style: Style| {
        if !tags.is_empty() {
            lines.push(Line::from(Span::styled(label.to_owned(), palette.muted)));
            lines.push(pill::row(
                tags.iter()
                    .map(|t| pill::pill(t, style, PillState::default()))
                    .collect(),
            ));
            lines.push(Line::default());
        }
    };
    group(
        "Deck tags",
        profile
            .tags
            .iter()
            .map(|t| deck_tag::deck_tag_label(t))
            .collect(),
        palette.accent,
    );
    group("Oracle tags", profile.oracle_tags.clone(), palette.muted);
    group(
        "Other tags",
        profile
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
        palette.warning,
    );
    f.render_widget(
        scrolled(
            Paragraph::new(lines).wrap(Wrap { trim: true }),
            view.scroll_of(Pane::Section(Section::Tags)),
            area,
        ),
        area,
    );
}

/// Average P/T, the Types chart, and the Roles bars under it. The Types chart shrinks first when the area is short.
fn draw_distributions(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &DeckView,
    metrics: &DeckMetrics,
    area: Rect,
) {
    let pt_lines = view
        .model
        .average_pt_line()
        .map(|pt| stat(palette, AVERAGE_PT, pt, area.width))
        .unwrap_or_default();
    let has_roles = metrics.card_role_bars().is_some_and(|r| !r.is_empty());
    let roles_min = if has_roles { ROLES_MIN_HEIGHT } else { 0 };
    let types_height = area
        .height
        .saturating_sub(measure::height_of(pt_lines.len()))
        .saturating_sub(roles_min)
        .min(CHART_HEIGHT);
    let [pt, types_area, roles_area] = Layout::vertical([
        Constraint::Length(measure::height_of(pt_lines.len())),
        Constraint::Length(types_height),
        Constraint::Fill(1),
    ])
    .areas(area);
    f.render_widget(Paragraph::new(pt_lines), pt);
    let types: Vec<Column> = metrics
        .type_bars()
        .into_iter()
        .map(|(label, count, _)| Column {
            label: label.to_uppercase(),
            count,
            style: palette.accent,
        })
        .collect();
    charts::columns(f, palette, "Types", &types, types_area);

    draw_roles(f, palette, view, metrics, roles_area);
}

/// The Roles bars, with the open role's tags and the open tag's description and Examples. The focused item stays in view.
fn draw_roles(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &DeckView,
    metrics: &DeckMetrics,
    roles_area: Rect,
) {
    let Some(roles) = metrics.card_role_bars() else {
        return;
    };
    let focused = view.focused_item();
    // Each label is a chip, padded by a space each side.
    let label_width = roles
        .iter()
        .map(|(l, _, _)| l.len().saturating_add(2))
        .max()
        .unwrap_or(0);
    let bar_width = usize::from(roles_area.width).saturating_sub(label_width.saturating_add(10));
    let mut lines = vec![Line::from(Span::styled("Roles", palette.muted))];
    let mut focus_line = 0;
    for (label, count, pct) in roles {
        let tags = metrics.card_role_tags.get(label);
        let is_focused = focused == Some(Item::Role(label.to_owned()));
        if is_focused {
            focus_line = lines.len();
        }
        // A role with tags to open is an accent chip; one with none is a grey chip that takes no key.
        let label_style = match (tags.is_some(), is_focused) {
            (true, true) => palette.accent.bold().reversed(),
            (true, false) => palette.accent.underlined(),
            (false, _) => palette.muted,
        };
        let mut line = charts::row(
            pill::pill(&label.to_uppercase(), label_style, PillState::default()),
            label_width,
            pct,
            bar_width,
            palette.accent,
            vec![Span::raw(count.to_string())],
        );
        line.spans
            .insert(0, Span::raw(if is_focused { "> " } else { "  " }));
        lines.push(line);
        if view.role_open.as_deref() != Some(label) {
            continue;
        }
        for tag in tags.into_iter().flatten() {
            let tag_focused = focused == Some(Item::Tag(tag.clone()));
            if tag_focused {
                focus_line = lines.len();
            }
            lines.push(Line::from(vec![
                Span::raw(if tag_focused { ">     " } else { "      " }),
                pill::pill(
                    tag,
                    palette.muted,
                    PillState {
                        picked: view.tag_open.as_deref() == Some(tag.as_str()),
                        focused: tag_focused,
                    },
                ),
            ]));
            if view.tag_open.as_deref() == Some(tag.as_str()) {
                let description = view
                    .tag_description(tag)
                    .unwrap_or("No description yet")
                    .to_owned();
                lines.push(Line::from(vec![
                    Span::raw("        "),
                    Span::styled(description, palette.text()),
                ]));
                let examples_focused = focused == Some(Item::Examples(tag.clone()));
                if examples_focused {
                    focus_line = lines.len();
                }
                lines.push(Line::from(vec![
                    Span::raw(if examples_focused {
                        ">       "
                    } else {
                        "        "
                    }),
                    pill::pill(
                        "Examples",
                        palette.accent,
                        PillState {
                            picked: false,
                            focused: examples_focused,
                        },
                    ),
                ]));
            }
        }
    }
    let height = usize::from(roles_area.height);
    let end = lines.len().saturating_sub(height);
    // On the last item the rows after it, roles with no tags, come into view too.
    let scroll = if focused.is_some() && focused == view.items().last().cloned() {
        end
    } else {
        focus_line.saturating_sub(height.saturating_sub(1)).min(end)
    };
    f.render_widget(
        Paragraph::new(lines).scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        roles_area,
    );
}

/// Lands and the average mana value, the Mana curve and Colors charts side by side or stacked when narrow, then the mana cost fulfillment rows.
fn draw_mana(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let metrics = view.model.charted();
    let mut summary = stat(palette, LANDS, view.model.lands_line(), area.width);
    if let Some(average) = view.model.average_mana_value_line() {
        summary.extend(stat(palette, AVERAGE_MANA_VALUE, average, area.width));
    }
    let Some(m) = metrics else {
        f.render_widget(Paragraph::new(summary), area);
        return;
    };

    let fulfillment = m.mana_balance_rows();
    let fulfillment_height = if fulfillment.is_empty() {
        0
    } else {
        measure::height_of(fulfillment.len()).saturating_add(1)
    };
    let side_by_side = area.width >= MANA_SIDE_BY_SIDE;
    let charts_max = if side_by_side {
        CHART_HEIGHT
    } else {
        CHART_HEIGHT.saturating_mul(2)
    };
    let summary_height = measure::height_of(summary.len());
    let charts_height = area
        .height
        .saturating_sub(summary_height)
        .saturating_sub(fulfillment_height)
        .min(charts_max);
    let [summary_area, charts_area, fulfillment_area] = Layout::vertical([
        Constraint::Length(summary_height),
        Constraint::Length(charts_height),
        Constraint::Fill(1),
    ])
    .areas(area);
    f.render_widget(Paragraph::new(summary), summary_area);

    let [curve_area, colors_area] = if side_by_side {
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(charts_area)
    } else {
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(charts_area)
    };
    let curve: Vec<Column> = m
        .mana_curve_bars()
        .into_iter()
        .zip(["0", "1", "2", "3", "4", "5", "6+"])
        .map(|((count, _), label)| Column {
            label: label.to_owned(),
            count,
            style: palette.accent,
        })
        .collect();
    charts::columns(f, palette, "Mana curve", &curve, curve_area);
    let colors: Vec<Column> = m
        .color_bars()
        .into_iter()
        .map(|(label, count, _)| Column {
            label: label.to_owned(),
            count,
            style: color_style(palette, label),
        })
        .collect();
    charts::columns(f, palette, "Colors", &colors, colors_area);

    if !fulfillment.is_empty() {
        let bar_width = usize::from(fulfillment_area.width).saturating_sub(16);
        let mut lines = vec![Line::from(Span::styled(
            "Mana cost fulfillment",
            palette.muted,
        ))];
        lines.extend(fulfillment.into_iter().map(|r| {
            let style = mana::letter_style(palette, r.label);
            let mark = if r.is_surplus { "✔ " } else { "  " };
            charts::row(
                Span::styled(r.label, style.bold()),
                1,
                r.fill_pct,
                bar_width,
                style,
                vec![
                    Span::styled(mark, palette.add),
                    Span::raw(format!("{}/{}", r.produced, r.consumed)),
                ],
            )
        }));
        f.render_widget(Paragraph::new(lines), fulfillment_area);
    }
}

fn draw_draw_odds(
    f: &mut Frame<'_>,
    palette: &Palette,
    metrics: &DeckMetrics,
    turn: u32,
    scroll: u16,
    area: Rect,
) {
    let (deck_size, buckets) = metrics.draw_odds_buckets();
    let heading = if turn == 0 {
        "Odds of ≥1 in opening hand".to_owned()
    } else {
        format!("Odds of ≥1 by turn {turn}")
    };
    let draws = OPENING_HAND.saturating_add(turn);
    let label_width = buckets.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    let bar_width = usize::from(area.width).saturating_sub(label_width.saturating_add(8));
    let mut lines = vec![
        Line::from(vec![
            Span::styled("- ", palette.accent.bold()),
            Span::raw(heading).bold(),
            Span::styled(" +", palette.accent.bold()),
        ]),
        Line::default(),
    ];
    lines.extend(buckets.into_iter().map(|(label, successes)| {
        let odds = draw_odds::p_at_least_one(deck_size, successes, draws);
        let pct = percent(odds);
        charts::row(
            Span::raw(label.to_uppercase()),
            label_width,
            pct,
            bar_width,
            palette.accent,
            vec![Span::raw(format!("{pct}%"))],
        )
    }));
    f.render_widget(scrolled(Paragraph::new(lines), scroll, area), area);
}

/// `paragraph` scrolled down `scroll` rows, no further than where its last row meets the bottom of `area`.
fn scrolled(paragraph: Paragraph<'static>, scroll: u16, area: Rect) -> Paragraph<'static> {
    let rows = measure::height_of(paragraph.line_count(area.width));
    paragraph.scroll((scroll.min(rows.saturating_sub(area.height)), 0))
}

/// A probability as a whole percentage, rounded.
fn percent(probability: f64) -> u32 {
    let clamped = (probability.clamp(0.0, 1.0) * 100.0).round();
    (0..=100_u32)
        .find(|&p| f64::from(p) >= clamped)
        .unwrap_or(100)
}

/// Each warning's message wrapped to the area, with its fix buttons under it. The focused button stays in view.
fn draw_warnings(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let focused = view.focused_item();
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut focus_line = 0;
    for (i, w) in view.model.warnings().iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled("! ", palette.warning.bold()),
            Span::styled(w.to_string(), palette.muted),
        ]));
        let fixes = deck_view::fix_buttons(w);
        if fixes.is_empty() {
            continue;
        }
        let mut buttons = vec![Span::raw("    ")];
        for (button, (label, _)) in fixes.iter().enumerate() {
            let is_focused = focused == Some(Item::Fix { warning: i, button });
            if is_focused {
                focus_line = lines.len();
            }
            buttons.push(pill::pill(
                label,
                palette.warning,
                PillState {
                    picked: is_focused,
                    focused: is_focused,
                },
            ));
            buttons.push(Span::raw(" "));
        }
        lines.push(Line::from(buttons));
    }
    let wrap = Wrap { trim: false };
    let focus_row = lines.iter().take(focus_line).fold(0_usize, |rows, line| {
        rows.saturating_add(
            Paragraph::new(line.clone())
                .wrap(wrap)
                .line_count(area.width),
        )
    });
    let scroll = focus_row.saturating_sub(usize::from(area.height).saturating_sub(1));
    f.render_widget(
        Paragraph::new(lines)
            .wrap(wrap)
            .scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        area,
    );
}

/// The mana style for a color chart label such as `green`; multicolor in the warning color, colorless muted.
fn color_style(palette: &Palette, label: &str) -> Style {
    match COLOR_LABELS.iter().position(|l| *l == label) {
        Some(i) => mana::style(palette, i),
        None if label == "multi" => palette.warning,
        None => palette.muted,
    }
}

/// The Clone dialog: its title and body, the name box, and Cancel and Save.
fn draw_clone(f: &mut Frame<'_>, palette: &Palette, clone: &CloneDialog) {
    let body = Paragraph::new(CLONE_BODY).wrap(Wrap { trim: true });
    let body_height = measure::height_of(body.line_count(DIALOG_WIDTH));
    let inner = panel::dialog(
        f,
        palette,
        f.area(),
        DIALOG_WIDTH,
        body_height.saturating_add(7),
    );
    let [title_area, body_area, input_area, _, buttons_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(body_height.saturating_add(1)),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    f.render_widget(
        Line::from(deck_view::Dialog::Clone.title())
            .style(palette.accent)
            .bold(),
        title_area,
    );
    f.render_widget(body, body_area);
    clone.name.draw(f, palette, input_area, "Name", !clone.busy);
    if !clone.busy {
        f.set_cursor_position(clone.name.cursor_position(input_area));
    }
    let save = if clone.busy { CLONE_BUSY } else { CLONE_SAVE };
    let save_style = if deck_view::can_clone(clone.name.value()) {
        palette.accent
    } else {
        palette.muted
    };
    f.render_widget(
        Line::from(vec![
            pill::pill(
                CANCEL,
                palette.accent,
                PillState {
                    picked: clone.focused == 0,
                    focused: false,
                },
            ),
            Span::raw("  "),
            pill::pill(
                save,
                save_style,
                PillState {
                    picked: clone.focused == 1,
                    focused: false,
                },
            ),
        ]),
        buttons_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_rounds_and_stays_in_range() {
        assert_eq!(percent(0.0), 0);
        assert_eq!(percent(0.394), 39);
        assert_eq!(percent(0.395), 40);
        assert_eq!(percent(1.0), 100);
        assert_eq!(percent(1.7), 100);
        assert_eq!(percent(-0.2), 0);
    }

    #[test]
    fn the_tab_window_keeps_the_shown_tab_and_its_neighbors() {
        let widths = [9, 8, 6, 15, 6, 11, 10];
        assert_eq!(tab_window(&widths, 3, 80), (0, 7), "all fit");
        assert_eq!(tab_window(&widths, 0, 30), (0, 3));
        assert_eq!(tab_window(&widths, 6, 30), (4, 7));
        assert_eq!(tab_window(&widths, 3, 30), (2, 5));
        assert_eq!(tab_window(&widths, 3, 4), (3, 4), "the shown tab stays");
        assert_eq!(tab_window(&[], 0, 10), (0, 0));
    }
}
