//! Mapping a deck request's reply to a deck screen's action.

use components::{TOAST_NORMAL, ToastOptions, Toasts};
use zwipe_client::ClientError;

/// Maps a deck request's reply to the screen's action: `landed` with the
/// value, `gone` when the server has no such deck, `failed` otherwise. The
/// request runner has already reported a failure and raised its toast.
pub(crate) fn settle<T, A>(
    reply: Result<T, ClientError>,
    landed: impl FnOnce(T) -> A,
    gone: A,
    failed: A,
) -> A {
    match reply {
        Ok(value) => landed(value),
        Err(ClientError::NotFound(_)) => gone,
        Err(_) => failed,
    }
}

/// [`settle`] after a quiet request: raises the toast for a gone deck
/// itself, since the runner raised none.
pub(crate) fn settle_quiet<T, A>(
    toast: Toasts,
    reply: Result<T, ClientError>,
    landed: impl FnOnce(T) -> A,
    gone: A,
    failed: A,
) -> A {
    if let Err(e @ ClientError::NotFound(_)) = &reply {
        toast.error(
            e.to_user_message(),
            ToastOptions::default().duration(TOAST_NORMAL),
        );
    }
    settle(reply, landed, gone, failed)
}
