//! The bridge from a zwipe-ui toast to the kit's toasts.

use dioxus_primitives::toast::{ToastOptions, Toasts};
use zwipe_ui::toast::{Toast, ToastKind};

/// Raises a zwipe-ui toast through the kit's toasts, keeping its kind, words
/// and duration.
pub fn raise(toast: Toasts, t: &Toast) {
    let options = ToastOptions::default().duration(t.duration);
    let text = t.text.clone();
    match t.kind {
        ToastKind::Info => toast.info(text, options),
        ToastKind::Success => toast.success(text, options),
        ToastKind::Warning => toast.warning(text, options),
        ToastKind::Error => toast.error(text, options),
    }
}
