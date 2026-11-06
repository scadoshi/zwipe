//! Signals owned by the runtime's root scope, for state a root task writes.

use dioxus::prelude::*;

/// A signal owned by [`ScopeId::ROOT`], made once from `init`.
///
/// A task started with `spawn_forever` runs in that scope, so it reads and
/// writes this signal as its owner does. A `use_signal` in `App` belongs to
/// the scope under it, and Dioxus warns on each use from the root task.
pub fn use_root_signal<T: 'static>(init: impl FnOnce() -> T) -> Signal<T> {
    use_hook(|| Signal::new_in_scope(init(), ScopeId::ROOT))
}

/// A component's own signal that root tasks also write, made once from
/// `init`. It is owned by [`ScopeId::ROOT`] like [`use_root_signal`], but
/// dropped with the calling component as a `use_signal` would be, so a root
/// task's `try_write` after the component is gone fails as it would there.
pub fn use_signal_for_root_tasks<T: 'static>(init: impl FnOnce() -> T) -> Signal<T> {
    let signal = use_root_signal(init);
    use_drop(move || signal.manually_drop());
    signal
}
