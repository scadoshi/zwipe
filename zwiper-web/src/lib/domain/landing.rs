//! Where a load of a signed-out route (Login, Register, Forgot password)
//! lands: on its form, or on Home when the stored session still refreshes.

/// How the one refresh of a stored session went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refresh {
    /// Not answered yet.
    Pending,
    /// The session is good: fresh already, or refreshed through the cookie.
    Live,
    /// The server turned the refresh down, or the session had run out.
    Rejected,
    /// The request failed on the way (network, server error).
    Failed,
}

/// What the signed-out route shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    /// Nothing but the themed background, while the refresh runs.
    Checking,
    /// Home, in place of the route.
    Home,
    /// The route's own screen.
    Form,
}

/// The landing for a load with a session `stored` (unexpired) or not, given
/// how its refresh went. No stored session shows the form at once.
#[must_use]
pub fn landing(stored: bool, refresh: Refresh) -> Landing {
    if !stored {
        return Landing::Form;
    }
    match refresh {
        Refresh::Pending => Landing::Checking,
        Refresh::Live => Landing::Home,
        Refresh::Rejected | Refresh::Failed => Landing::Form,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_stored_session_shows_the_form_without_a_check() {
        for refresh in [
            Refresh::Pending,
            Refresh::Live,
            Refresh::Rejected,
            Refresh::Failed,
        ] {
            assert_eq!(landing(false, refresh), Landing::Form);
        }
    }

    #[test]
    fn a_stored_session_waits_for_its_refresh() {
        assert_eq!(landing(true, Refresh::Pending), Landing::Checking);
    }

    #[test]
    fn a_live_session_goes_home() {
        assert_eq!(landing(true, Refresh::Live), Landing::Home);
    }

    #[test]
    fn a_rejected_or_failed_refresh_shows_the_form() {
        assert_eq!(landing(true, Refresh::Rejected), Landing::Form);
        assert_eq!(landing(true, Refresh::Failed), Landing::Form);
    }
}
