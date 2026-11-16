//! A count of component renders in debug builds, for tests that pin how much a change
//! re-renders. A component names itself with [`count_render!`](crate::count_render); a test
//! resets the count, makes its change and reads it back with [`renders`]. The count is per
//! thread, so tests running side by side don't see each other's renders, and a release build
//! counts nothing.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static COUNTS: RefCell<HashMap<&'static str, usize>> = RefCell::new(HashMap::new());
}

/// Counts one render of `name` on this thread. [`count_render!`](crate::count_render) calls it.
#[doc(hidden)]
pub fn bump(name: &'static str) {
    COUNTS.with(|counts| {
        let mut counts = counts.borrow_mut();
        let count = counts.entry(name).or_default();
        *count = count.saturating_add(1);
    });
}

/// How many times `name` rendered on this thread since the last [`reset`].
#[must_use]
pub fn renders(name: &str) -> usize {
    COUNTS.with(|counts| counts.borrow().get(name).copied().unwrap_or(0))
}

/// Forgets every count on this thread.
pub fn reset() {
    COUNTS.with(|counts| counts.borrow_mut().clear());
}

/// Counts a render of the component it opens, in a debug build only.
#[macro_export]
macro_rules! count_render {
    ($name:literal) => {
        #[cfg(debug_assertions)]
        $crate::render_count::bump($name);
    };
}
