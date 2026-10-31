//! The notice shown when the server no longer accepts this version. The app draws it over the stack while `Shared::update_required` holds, and the event loop lets only Esc and `q` through, to quit.

pub(crate) mod draw;
