//! Account: the Account and About lists with their buttons, the action bar, and the sheets and dialogs over them.

use chrono::Utc;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::user::User;
use zwipe_ui::screens::account::{
    self, CANCEL, CHANGE, CHANGELOG, DELETING, Dialog, Field, LOG_OUT, MORE, MORE_TITLE, MoreItem,
    PASSWORD_MASK, Row, State, VERIFIED, VIEW, WEBSITE_LINK,
};

use super::{AccountView, DeleteDialog, DeleteFocus, Form, Overlay, Stop, label};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::{VERSION, login};
use crate::inbound::widgets::action_bar;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::button::{self, Button};
use crate::inbound::widgets::sheet::{self, Sheet};
use crate::inbound::widgets::text_input::{BOX_HEIGHT, TextInput};

/// The lists are at most this wide, in cells.
const WIDTH: u16 = 72;

/// The labels take this many cells.
const LABEL_WIDTH: usize = 18;

/// A Change sheet's fields are at most this wide, in cells.
const FORM_WIDTH: u16 = 48;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &AccountView) {
    let palette = &shared.palette;
    let bar = action_bar::line(
        palette,
        &[
            ("j/k", "move"),
            ("Enter", "select"),
            ("i", "about row"),
            ("L", "log out"),
            ("m", "more"),
            ("Esc", "back"),
        ],
    );
    let bar = action_bar::with_help(palette, bar, action_bar::Hint::Question).centered();
    let area = f.area();
    let bar_height = action_bar::height(&bar, area.width);
    let width = WIDTH.min(area.width);
    let lines = lines(palette, view, usize::from(width));
    let row = [Button::new(LOG_OUT).danger(), Button::new(MORE)];
    let [body, buttons, keys] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(bar_height),
    ])
    .areas(area);
    let body = body.centered_horizontally(Constraint::Length(width));
    f.render_widget(Paragraph::new(lines), body);
    button::draw_row(f, palette, buttons, &row, None);
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    match &view.overlay {
        Some(Overlay::Form(form)) => draw_form(f, palette, &view.model, form),
        Some(Overlay::ConfirmPassword { form, alert }) => {
            draw_form(f, palette, &view.model, form);
            alert.draw(f, palette);
        }
        Some(Overlay::LogOut(alert)) => alert.draw(f, palette),
        Some(Overlay::More(delete)) => {
            more_sheet(palette).draw(f, palette, f.area());
            if let Some(delete) = delete {
                draw_delete(f, palette, &view.model, delete);
            }
        }
        None => {}
    }
}

/// A row: its label (with its `[?]` when it has a topic), its value, and its buttons on the right.
fn row(
    width: usize,
    label: &str,
    about: Option<Vec<Span<'static>>>,
    value: Vec<Span<'static>>,
    buttons: Vec<Span<'static>>,
) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{label:<LABEL_WIDTH$}"))];
    if let Some(about) = about {
        spans.pop();
        spans.push(Span::raw(format!("{label} ")));
        spans.extend(about);
        let used = label
            .chars()
            .count()
            .saturating_add(1)
            .saturating_add(usize::from(button::ABOUT_WIDTH));
        spans.push(Span::raw(" ".repeat(LABEL_WIDTH.saturating_sub(used))));
    }
    spans.extend(value);
    let used: usize = spans.iter().map(Span::width).sum();
    let tail: usize = buttons.iter().map(Span::width).sum();
    spans.push(Span::raw(
        " ".repeat(width.saturating_sub(used).saturating_sub(tail)),
    ));
    spans.extend(buttons);
    Line::from(spans)
}

/// A row's `[?]`, focused while one of `stops` has the focus, since `i` then opens its topic.
fn about(palette: &Palette, view: &AccountView, stops: &[Stop]) -> Vec<Span<'static>> {
    let focused =
        view.overlay.is_none() && view.focused().is_some_and(|stop| stops.contains(&stop));
    button::about(palette, focused)
}

/// A button on a row, highlighted while focused and muted while disabled.
fn button(
    palette: &Palette,
    view: &AccountView,
    stop: Stop,
    label: &str,
    disabled: bool,
) -> Vec<Span<'static>> {
    let focused = view.focused() == Some(stop) && view.overlay.is_none();
    button::spans(palette, &Button::new(label).disabled(disabled), focused)
}

/// The screen's lines, top to bottom: the model's rows under their lists' titles, a blank line between the lists. Nothing with no user.
fn lines(palette: &Palette, view: &AccountView, width: usize) -> Vec<Line<'static>> {
    let model = &view.model;
    let Some(user) = model.user() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let mut list = None;
    for at in model.rows() {
        if list != Some(at.list()) {
            if list.is_some() {
                lines.push(Line::default());
            }
            list = Some(at.list());
            lines.push(section(palette, at.list()));
        }
        lines.extend(row_lines(palette, view, user, at, width));
    }
    lines
}

/// The lines `at` draws as: its row, and the Pending row's note under it.
fn row_lines(
    palette: &Palette,
    view: &AccountView,
    user: &User,
    at: Row,
    width: usize,
) -> Vec<Line<'static>> {
    let model = &view.model;
    let change = |stop| button(palette, view, stop, CHANGE, /* disabled */ false);
    let marker = |stops: &[Stop]| Some(about(palette, view, stops));
    let (marker, value, buttons) = match at {
        Row::Username => (
            None,
            vec![Span::raw(user.username.to_string())],
            change(Stop::ChangeUsername),
        ),
        Row::Email => {
            let badge = if model.badge() == VERIFIED {
                palette.add
            } else {
                palette.error
            };
            (
                marker(&[Stop::ChangeEmail]),
                vec![
                    Span::raw(format!("{} ", user.email)),
                    Span::styled(model.badge(), badge),
                ],
                change(Stop::ChangeEmail),
            )
        }
        Row::Pending => return pending_lines(palette, model, width),
        Row::Verification => (
            marker(&[Stop::Resend, Stop::Check]),
            Vec::new(),
            [
                button(
                    palette,
                    view,
                    Stop::Resend,
                    &model.resend_label(),
                    !model.can_resend(),
                ),
                vec![Span::raw(" ")],
                button(palette, view, Stop::Check, "↻", model.is_checking()),
            ]
            .concat(),
        ),
        Row::Password => (
            None,
            vec![Span::raw(PASSWORD_MASK)],
            change(Stop::ChangePassword),
        ),
        Row::Website => (
            marker(&[Stop::Website]),
            Vec::new(),
            button(palette, view, Stop::Website, WEBSITE_LINK, false),
        ),
        Row::PrivacyPolicy => (
            None,
            Vec::new(),
            button(
                palette,
                view,
                Stop::Privacy,
                VIEW,
                /* disabled */ false,
            ),
        ),
        Row::Version => (
            marker(&[Stop::Changelog]),
            vec![Span::raw(account::version_label(VERSION))],
            button(
                palette,
                view,
                Stop::Changelog,
                CHANGELOG,
                /* disabled */ false,
            ),
        ),
    };
    vec![row(width, at.label(), marker, value, buttons)]
}

/// The Pending row and the note on its link under it, while an address waits on its link.
fn pending_lines(palette: &Palette, model: &State, width: usize) -> Vec<Line<'static>> {
    let Some(pending) = model.pending(Utc::now()) else {
        return Vec::new();
    };
    let mut lines = vec![row(
        width,
        Row::Pending.label(),
        None,
        Vec::new(),
        vec![Span::raw(pending.email)],
    )];
    if let Some(note) = pending.note {
        lines.push(Line::from(note).style(palette.muted).right_aligned());
    }
    lines
}

/// A muted, bold section title.
fn section(palette: &Palette, title: &str) -> Line<'static> {
    Line::from(title.to_owned()).style(palette.muted).bold()
}

/// The More sheet: one danger button, Delete account.
fn more_sheet(palette: &Palette) -> Sheet {
    let delete = MoreItem::DeleteAccount.label();
    Sheet::new(
        MORE_TITLE,
        vec![
            Line::from(button::spans(
                palette,
                &Button::new(delete).danger(),
                /* focused */ true,
            ))
            .centered(),
        ],
    )
    .with_footer("Enter", delete)
    .with_footer("Esc", "Close")
}

/// A Change sheet: its fields with their error lines, then Save changes, and its footer keys. Esc goes back.
fn draw_form(f: &mut Frame<'_>, palette: &Palette, model: &State, form: &Form) {
    let heights: Vec<u16> = form.inputs.iter().map(TextInput::height).collect();
    let fields = heights.iter().fold(0_u16, |sum, h| sum.saturating_add(*h));
    let busy = model.is_saving(form.sheet);
    let row = [Button::new(model.save_label(form.sheet)).disabled(busy)];
    let mut footer = vec![("Tab", "next"), ("Enter", "save"), ("Esc", "back")];
    if form.field() == Some(Field::NewPassword) {
        footer.insert(1, ("F2", "password rules"));
    }
    let buttons_height = 1;
    let inner = sheet::frame(
        f,
        palette,
        f.area(),
        form.sheet.title(),
        fields.saturating_add(1).saturating_add(buttons_height),
        false,
        &footer,
    );
    let inner = inner.centered_horizontally(Constraint::Length(FORM_WIDTH.min(inner.width)));
    let [fields_area, _, buttons] = Layout::vertical([
        Constraint::Length(fields),
        Constraint::Length(1),
        Constraint::Length(buttons_height),
    ])
    .areas(inner);
    let areas = Layout::vertical(heights.iter().map(|h| Constraint::Length(*h))).split(fields_area);
    for (i, ((input, field), area)) in form
        .inputs
        .iter()
        .zip(form.sheet.fields())
        .zip(areas.iter())
        .enumerate()
    {
        let focused = i == form.focus;
        input.draw(f, palette, *area, label(*field), focused);
        if focused && !busy {
            f.set_cursor_position(input.cursor_position(*area));
        }
    }
    button::draw_row(f, palette, buttons, &row, None);
}

/// The Delete account dialog over the More sheet: the warning, a password box, and Cancel and Delete.
fn draw_delete(f: &mut Frame<'_>, palette: &Palette, model: &State, delete: &DeleteDialog) {
    let busy = model.is_deleting();
    let label = if busy {
        DELETING.to_owned()
    } else {
        model.delete_label()
    };
    let dialog = Dialog::DeleteAccount;
    let mut alert = AlertDialog::new(dialog.title(), dialog.body())
        .with_button(CANCEL, None)
        .with_danger_button(&label, None);
    alert.set_button(0, CANCEL, busy);
    alert.set_button(1, &label, !model.can_delete());
    alert.focused = match delete.focus {
        DeleteFocus::Delete => 1,
        _ => 0,
    };
    let buttons_focused = matches!(delete.focus, DeleteFocus::Cancel | DeleteFocus::Delete);
    if !buttons_focused {
        alert.focused = usize::MAX;
    }
    alert.draw_with(f, palette, BOX_HEIGHT, |f: &mut Frame<'_>, area: Rect| {
        delete
            .password
            .draw(f, palette, area, "", delete.focus == DeleteFocus::Password);
        login::draw::show_button(
            f,
            palette,
            area,
            &delete.password,
            delete.focus == DeleteFocus::ShowPassword,
        );
        if delete.focus == DeleteFocus::Password && !busy {
            f.set_cursor_position(delete.password.cursor_position(area));
        }
    });
}
