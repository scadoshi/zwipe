//! Account: the Account and About lists with their buttons, the action bar, and the sheets and dialogs over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{AccountView, DeleteDialog, DeleteFocus, Form, FormKind, Overlay, Stop, pending_note};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::{VERSION, login};
use crate::inbound::widgets::action_bar;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::button::{self, Button};
use crate::inbound::widgets::sheet::{self, Sheet};
use crate::inbound::widgets::text_input::{BOX_HEIGHT, MASK, TextInput};

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
    let row = [Button::new("Log out").danger(), Button::new("More")];
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
        Some(Overlay::Form(form)) => draw_form(f, palette, form),
        Some(Overlay::ConfirmPassword { form, alert }) => {
            draw_form(f, palette, form);
            alert.draw(f, palette);
        }
        Some(Overlay::LogOut(alert)) => alert.draw(f, palette),
        Some(Overlay::More(delete)) => {
            more_sheet(palette).draw(f, palette, f.area());
            if let Some(delete) = delete {
                draw_delete(f, palette, delete);
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

/// The screen's lines, top to bottom.
fn lines(palette: &Palette, view: &AccountView, width: usize) -> Vec<Line<'static>> {
    let mut lines = account_lines(palette, view, width);
    lines.push(Line::default());
    lines.extend(about_lines(palette, view, width));
    lines
}

/// A muted, bold section title.
fn section(palette: &Palette, title: &str) -> Line<'static> {
    Line::from(title.to_owned()).style(palette.muted).bold()
}

/// The Account section: username, email, any pending email, verification and password.
fn account_lines(palette: &Palette, view: &AccountView, width: usize) -> Vec<Line<'static>> {
    let user = &view.user;
    let mut lines = vec![
        section(palette, "Account"),
        row(
            width,
            "Username",
            None,
            vec![Span::raw(user.username.to_string())],
            button(
                palette,
                view,
                Stop::ChangeUsername,
                "Change",
                /* disabled */ false,
            ),
        ),
        row(
            width,
            "Email",
            Some(about(palette, view, &[Stop::ChangeEmail])),
            vec![
                Span::raw(format!("{} ", user.email)),
                if view.verified() {
                    Span::styled("Verified", palette.add)
                } else {
                    Span::styled("Unverified", palette.error)
                },
            ],
            button(
                palette,
                view,
                Stop::ChangeEmail,
                "Change",
                /* disabled */ false,
            ),
        ),
    ];
    if let Some(pending) = &user.pending_email {
        lines.push(row(
            width,
            "Pending",
            None,
            Vec::new(),
            vec![Span::raw(pending.to_string())],
        ));
        if let Some(note) = pending_note(user) {
            lines.push(Line::from(note).style(palette.muted).right_aligned());
        }
    }
    if !view.verified() {
        lines.push(row(
            width,
            "Verification",
            Some(about(palette, view, &[Stop::Resend, Stop::Check])),
            Vec::new(),
            [
                button(
                    palette,
                    view,
                    Stop::Resend,
                    &view.resend_label(),
                    !view.can_resend(),
                ),
                vec![Span::raw(" ")],
                button(palette, view, Stop::Check, "↻", view.checking),
            ]
            .concat(),
        ));
    }
    lines.push(row(
        width,
        "Password",
        None,
        vec![Span::raw(MASK.to_string().repeat(7))],
        button(
            palette,
            view,
            Stop::ChangePassword,
            "Change",
            /* disabled */ false,
        ),
    ));
    lines
}

/// The About section: website, privacy policy and version.
fn about_lines(palette: &Palette, view: &AccountView, width: usize) -> Vec<Line<'static>> {
    vec![
        section(palette, "About"),
        row(
            width,
            "Website",
            Some(about(palette, view, &[Stop::Website])),
            Vec::new(),
            button(palette, view, Stop::Website, "zwipe.net \u{2197}", false),
        ),
        row(
            width,
            "Privacy policy",
            None,
            Vec::new(),
            button(
                palette,
                view,
                Stop::Privacy,
                "View",
                /* disabled */ false,
            ),
        ),
        row(
            width,
            "Version",
            Some(about(palette, view, &[Stop::Changelog])),
            vec![Span::raw(format!("v{VERSION}"))],
            button(
                palette,
                view,
                Stop::Changelog,
                "Changelog",
                /* disabled */ false,
            ),
        ),
    ]
}

/// The More sheet: one danger button, Delete account.
fn more_sheet(palette: &Palette) -> Sheet {
    Sheet::new(
        "More",
        vec![
            Line::from(button::spans(
                palette,
                &Button::new("Delete account").danger(),
                /* focused */ true,
            ))
            .centered(),
        ],
    )
    .with_footer("Enter", "Delete account")
    .with_footer("Esc", "Close")
}

/// A Change sheet: its fields with their error lines, then Save changes, and its footer keys. Esc goes back.
fn draw_form(f: &mut Frame<'_>, palette: &Palette, form: &Form) {
    let heights: Vec<u16> = form.inputs.iter().map(TextInput::height).collect();
    let fields = heights.iter().fold(0_u16, |sum, h| sum.saturating_add(*h));
    let save = if form.busy {
        "Saving..."
    } else {
        "Save changes"
    };
    let row = [Button::new(save).disabled(form.busy)];
    let mut footer = vec![("Tab", "next"), ("Enter", "save"), ("Esc", "back")];
    if form.kind == FormKind::Password && form.focus == 1 {
        footer.insert(1, ("F2", "password rules"));
    }
    let buttons_height = 1;
    let inner = sheet::frame(
        f,
        palette,
        f.area(),
        form.kind.title(),
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
    for (i, ((input, (label, _)), area)) in form
        .inputs
        .iter()
        .zip(form.kind.fields())
        .zip(areas.iter())
        .enumerate()
    {
        let focused = i == form.focus;
        input.draw(f, palette, *area, *label, focused);
        if focused && !form.busy {
            f.set_cursor_position(input.cursor_position(*area));
        }
    }
    button::draw_row(f, palette, buttons, &row, None);
}

/// The Delete account dialog over the More sheet: the warning, a password box, and Cancel and Delete.
fn draw_delete(f: &mut Frame<'_>, palette: &Palette, delete: &DeleteDialog) {
    let disabled = delete.busy || delete.wait().is_some();
    let mut alert = AlertDialog::new(
        "Delete account",
        "This will permanently delete your account, all decks, and all card data. This cannot be undone.",
    )
    .with_button("Cancel", None)
    .with_danger_button(&delete.label(), None);
    alert.set_button(0, "Cancel", delete.busy);
    alert.set_button(1, &delete.label(), disabled);
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
        if delete.focus == DeleteFocus::Password && !delete.busy {
            f.set_cursor_position(delete.password.cursor_position(area));
        }
    });
}
