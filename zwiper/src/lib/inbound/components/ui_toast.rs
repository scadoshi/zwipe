//! Raises a toast zwipe-ui decided on through the app's toaster.

use components::{ToastOptions, Toasts};
use zwipe_ui::toast::{Toast, ToastKind};

/// Raises `toast` with its kind, words, band and whether it carries over to the next screen.
pub(crate) fn raise(toasts: Toasts, toast: Toast) {
    let options = ToastOptions::default().duration(toast.duration);
    let kind = toast.kind;
    let title = toast.into_title();
    match kind {
        ToastKind::Info => toasts.info(title, options),
        ToastKind::Success => toasts.success(title, options),
        ToastKind::Warning => toasts.warning(title, options),
        ToastKind::Error => toasts.error(title, options),
    }
}
