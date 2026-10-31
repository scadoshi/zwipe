//! Account's keys: the row buttons, the action bar's letters, and the sheets and dialogs over the screen.

use crossterm::event::KeyCode;
use zwipe_core::domain::site::WEB_BASE;
use zwipe_core::domain::user::hints::HINT_PROFILE;

use super::{
    AccountChange, AccountView, DeleteDialog, DeleteFocus, Form, FormKind, Overlay, Request, Stop,
    confirm_password_alert, log_out_alert,
};
use crate::inbound::app::Context;
use crate::inbound::screens::{Password, Request as AppRequest, changelog, privacy};
use crate::inbound::widgets::alert::AlertChoice;
use crate::inbound::widgets::hint::{self, HintTopic};
use crate::inbound::widgets::{browser, help};

pub(crate) fn handle(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    match view.overlay.take() {
        Some(Overlay::Form(form)) => view.overlay = handle_form(form, code, cx),
        Some(Overlay::ConfirmPassword {
            mut form,
            mut alert,
        }) => {
            view.overlay = match alert.handle_key(code) {
                Some(AlertChoice::Pressed(1)) => {
                    if form.attempt() {
                        form.busy = true;
                        cx.request(Request::Change(form.change()).into());
                    }
                    Some(Overlay::Form(form))
                }
                Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => Some(Overlay::Form(form)),
                None => Some(Overlay::ConfirmPassword { form, alert }),
            };
        }
        Some(Overlay::LogOut(mut alert)) => match alert.handle_key(code) {
            Some(AlertChoice::Pressed(1)) => cx.request(AppRequest::SignOut),
            Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => {}
            None => view.overlay = Some(Overlay::LogOut(alert)),
        },
        Some(Overlay::More(Some(delete))) => {
            view.overlay = Some(Overlay::More(handle_delete(delete, code, cx)));
        }
        Some(Overlay::More(None)) => {
            view.overlay = match code {
                KeyCode::Esc | KeyCode::Char('q') => None,
                KeyCode::Enter | KeyCode::Char('d') => {
                    Some(Overlay::More(Some(DeleteDialog::new())))
                }
                _ => Some(Overlay::More(None)),
            };
        }
        None => handle_screen(view, code, cx),
    }
}

/// Keys on the screen itself.
fn handle_screen(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => view.next(),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => view.previous(),
        KeyCode::Enter => {
            if let Some(stop) = view.focused() {
                activate(view, stop, cx);
            }
        }
        KeyCode::Char('i') => {
            if let Some(topic) = view.focused().and_then(Stop::topic) {
                hint::topic(cx, topic);
            }
        }
        KeyCode::Char('?') => hint::reopen(cx, HINT_PROFILE),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('L') => view.overlay = Some(Overlay::LogOut(log_out_alert())),
        KeyCode::Char('m') => view.overlay = Some(Overlay::More(None)),
        _ => {}
    }
}

fn activate(view: &mut AccountView, stop: Stop, cx: &mut Context<'_>) {
    match stop {
        Stop::ChangeUsername => view.overlay = Some(Overlay::Form(Form::new(FormKind::Username))),
        Stop::ChangeEmail => view.overlay = Some(Overlay::Form(Form::new(FormKind::Email))),
        Stop::ChangePassword => view.overlay = Some(Overlay::Form(Form::new(FormKind::Password))),
        Stop::Resend => {
            if view.can_resend() {
                view.sending = true;
                cx.request(Request::Resend.into());
            }
        }
        Stop::Check => {
            if !view.checking {
                view.checking = true;
                cx.request(Request::CheckVerification.into());
            }
        }
        Stop::Website => browser::open(cx, WEB_BASE),
        Stop::Privacy => privacy::open(cx),
        Stop::Changelog => changelog::open(cx),
    }
}

/// Keys in a Change sheet. Returns what stays open: the sheet, its password confirm, or nothing.
fn handle_form(mut form: Form, code: KeyCode, cx: &mut Context<'_>) -> Option<Overlay> {
    if form.busy {
        return Some(Overlay::Form(form));
    }
    match code {
        KeyCode::Esc => return None,
        KeyCode::Tab | KeyCode::Down => form.next_field(),
        KeyCode::BackTab | KeyCode::Up => form.previous_field(),
        KeyCode::F(2) if form.kind == FormKind::Password && form.focus == 1 => {
            hint::topic(cx, HintTopic::Password);
        }
        KeyCode::Enter if !form.on_last_field() => form.next_field(),
        KeyCode::Enter => {
            if !form.attempt() {
                return Some(Overlay::Form(form));
            }
            if form.kind == FormKind::Password {
                return Some(Overlay::ConfirmPassword {
                    form,
                    alert: confirm_password_alert(),
                });
            }
            form.busy = true;
            cx.request(Request::Change(form.change()).into());
        }
        other => {
            let at = form.focus;
            if let Some(input) = form.inputs.get_mut(at)
                && input.handle_key(other)
            {
                if !input.is_empty()
                    && let Some(touched) = form.touched.get_mut(at)
                {
                    *touched = true;
                }
                form.validate();
            }
        }
    }
    Some(Overlay::Form(form))
}

/// Keys in the Delete account dialog. Returns it while it stays open over the More sheet.
fn handle_delete(
    mut delete: DeleteDialog,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<DeleteDialog> {
    if delete.busy {
        return Some(delete);
    }
    let order = [
        DeleteFocus::Password,
        DeleteFocus::ShowPassword,
        DeleteFocus::Cancel,
        DeleteFocus::Delete,
    ];
    let at = order.iter().position(|f| *f == delete.focus).unwrap_or(0);
    match code {
        KeyCode::Esc => return None,
        KeyCode::Tab | KeyCode::Down => {
            delete.focus = order
                .get(at.saturating_add(1) % order.len())
                .copied()
                .unwrap_or(DeleteFocus::Password);
        }
        KeyCode::BackTab | KeyCode::Up => {
            let back = at
                .checked_sub(1)
                .unwrap_or_else(|| order.len().saturating_sub(1));
            delete.focus = order.get(back).copied().unwrap_or(DeleteFocus::Password);
        }
        KeyCode::Enter => match delete.focus {
            DeleteFocus::Cancel => return None,
            DeleteFocus::ShowPassword => {
                let masked = delete.password.is_masked();
                delete.password.set_masked(!masked);
            }
            DeleteFocus::Password | DeleteFocus::Delete => {
                // The countdown alone gates it; an empty password is the server's to refuse.
                if delete.wait().is_none() {
                    delete.busy = true;
                    cx.request(
                        Request::Change(AccountChange::Delete {
                            password: Password(delete.password.value().to_owned()),
                        })
                        .into(),
                    );
                }
            }
        },
        other if delete.focus == DeleteFocus::Password => {
            delete.password.handle_key(other);
        }
        _ => {}
    }
    Some(delete)
}
