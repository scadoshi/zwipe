//! Raises a toast zwipe-ui decided on through the app's toaster.

use dioxus_primitives::toast::{ToastOptions, Toasts};
use zwipe_ui::toast::{Toast, ToastKind};

/// Raises `toast` with its kind, words and band.
pub(crate) fn raise(toasts: Toasts, toast: Toast) {
    let options = ToastOptions::default().duration(toast.duration);
    match toast.kind {
        ToastKind::Info => toasts.info(toast.text, options),
        ToastKind::Success => toasts.success(toast.text, options),
        ToastKind::Warning => toasts.warning(toast.text, options),
        ToastKind::Error => toasts.error(toast.text, options),
    }
}
