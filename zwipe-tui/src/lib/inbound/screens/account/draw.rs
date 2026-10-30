//! The account screen: the user's details, verification, the about rows, and the forms and dialogs over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::site::WEB_BASE;

use super::{AccountRow, AccountView, Dialog, Form, FormKind, pending_note};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::VERSION;
use crate::inbound::widgets::measure::height_of;
use crate::inbound::widgets::text_input::MASK;
use crate::inbound::widgets::{action_bar, panel};

/// The labels take this many cells before their values.
const LABEL_WIDTH: usize = 14;

/// A dialog is at most this wide inside its panel, in cells.
const DIALOG_WIDTH: u16 = 56;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &AccountView) {
    let palette = &shared.palette;
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    f.render_widget(
        Line::from(Span::styled("Account", palette.accent).bold()),
        header,
    );
    f.render_widget(
        Paragraph::new(lines(palette, view)).block(Block::bordered().border_style(palette.border)),
        body,
    );
    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    f.render_widget(
        action_bar::line(
            palette,
            &[
                ("j/k", "move"),
                ("Enter", "open"),
                ("Esc", "home"),
                ("q", "quit"),
            ],
        ),
        keys,
    );

    match &view.dialog {
        Some(Dialog::Form(form)) => draw_form(f, palette, form),
        Some(Dialog::ConfirmPassword { alert, .. } | Dialog::LogOut(alert)) => {
            alert.draw(f, palette);
        }
        None => {}
    }
}

/// The screen's lines, top to bottom, with the selected row marked.
fn lines(palette: &Palette, view: &AccountView) -> Vec<Line<'static>> {
    let selected = view.selected();
    let row = |row: AccountRow, label: &str, value: Vec<Span<'static>>, action: &str| {
        let marked = selected == Some(row);
        let mut spans = vec![
            Span::raw(if marked { "> " } else { "  " }),
            Span::styled(format!("{label:<LABEL_WIDTH$}"), palette.muted),
        ];
        spans.extend(value);
        if !action.is_empty() {
            spans.push(Span::styled(format!("  {action}"), palette.accent));
        }
        let line = Line::from(spans);
        if marked { line.bold() } else { line }
    };
    let plain = |label: &str, value: String| {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{label:<LABEL_WIDTH$}"), palette.muted),
            Span::raw(value),
        ])
    };
    let user = &view.user;

    let mut lines = vec![
        Line::from(Span::styled("Account", palette.accent)),
        row(
            AccountRow::Username,
            "Username",
            vec![Span::raw(user.username.to_string())],
            "Change",
        ),
        row(
            AccountRow::Email,
            "Email",
            vec![
                Span::raw(user.email.to_string()),
                Span::raw("  "),
                if view.verified() {
                    Span::styled("Verified", palette.add)
                } else {
                    Span::styled("Unverified", palette.warning)
                },
            ],
            "Change",
        ),
    ];
    if let Some(pending) = &user.pending_email {
        lines.push(plain("Pending", pending.to_string()));
        if let Some(note) = pending_note(user) {
            lines.push(plain("", note).style(palette.muted));
        }
    }
    if !view.verified() {
        let resend = view
            .resend_wait()
            .map_or_else(|| "Resend".to_owned(), |s| format!("Resend in {s}s"));
        lines.push(row(AccountRow::Resend, "Verification", Vec::new(), &resend));
        lines.push(row(AccountRow::Check, "", Vec::new(), "Check"));
    }
    lines.extend([
        row(
            AccountRow::Password,
            "Password",
            vec![Span::raw(MASK.to_string().repeat(7))],
            "Change",
        ),
        Line::default(),
        Line::from(Span::styled("About", palette.accent)),
        plain("Website", WEB_BASE.to_owned()),
        plain("Version", format!("v{VERSION}")),
        Line::default(),
        row(AccountRow::LogOut, "Log out", Vec::new(), ""),
        row(AccountRow::Delete, "Delete account", Vec::new(), "").style(palette.error),
    ]);
    lines
}

fn draw_form(f: &mut Frame<'_>, palette: &Palette, form: &Form) {
    let fields = form.kind.fields();
    let intro = match form.kind {
        FormKind::Delete => Some(
            "This will permanently delete your account, all decks, and all card data. This cannot be undone.",
        ),
        _ => None,
    };
    let intro = intro.map(|text| Paragraph::new(text).wrap(Wrap { trim: true }));
    let intro_height = intro.as_ref().map_or(0, |p| {
        height_of(p.line_count(DIALOG_WIDTH)).saturating_add(1)
    });
    let error = form.error.as_ref().map(|e| {
        Paragraph::new(Line::from(e.clone()).style(palette.error)).wrap(Wrap { trim: true })
    });
    let error_height = error
        .as_ref()
        .map_or(0, |p| height_of(p.line_count(DIALOG_WIDTH)));
    let fields_height = u16::try_from(fields.len()).unwrap_or(0).saturating_mul(3);
    let height = intro_height
        .saturating_add(fields_height)
        .saturating_add(error_height)
        .saturating_add(2);

    let inner = panel::draw(f, palette, f.area(), DIALOG_WIDTH, height.saturating_add(1));
    let [title_area, rest] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
    f.render_widget(Line::from(form.kind.title()).bold(), title_area);
    let [intro_area, fields_area, error_area, _, keys_area] = Layout::vertical([
        Constraint::Length(intro_height),
        Constraint::Length(fields_height),
        Constraint::Length(error_height),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(rest);

    if let Some(intro) = intro {
        f.render_widget(intro, intro_area);
    }
    let boxes = Layout::vertical(fields.iter().map(|_| Constraint::Length(3))).split(fields_area);
    for (i, ((label, _), (input, area))) in fields
        .iter()
        .zip(form.inputs.iter().zip(boxes.iter()))
        .enumerate()
    {
        let focused = i == form.focus;
        input.draw(f, palette, *area, label, focused);
        if focused && !form.busy {
            f.set_cursor_position(input.cursor_position(*area));
        }
    }
    if let Some(error) = error {
        f.render_widget(error, error_area);
    }
    f.render_widget(form_keys(palette, form), keys_area);
}

fn form_keys(palette: &Palette, form: &Form) -> Line<'static> {
    if form.busy {
        let busy = if form.kind == FormKind::Delete {
            "Deleting..."
        } else {
            "Saving..."
        };
        return Line::from(busy).style(palette.muted).bold();
    }
    let submit = match (form.kind, form.delete_wait()) {
        (FormKind::Delete, Some(wait)) => format!("Delete ({wait})"),
        (FormKind::Delete, None) => "Delete".to_owned(),
        _ => "Save changes".to_owned(),
    };
    action_bar::line(
        palette,
        &[
            ("Tab", "next field"),
            ("Enter", submit.as_str()),
            ("Esc", "back"),
        ],
    )
}
