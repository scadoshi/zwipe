//! Pre-auth funnel event POST (no auth: there is no user yet).

use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::HttpAnonymousEvent;
use zwipe_core::http::endpoints::metrics::RecordAnonymousEvent;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Posts a pre-auth funnel event, stamped with [`ClientPlatform::CURRENT`] unless it already names a platform.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn record_anonymous_event(
        &self,
        event: &HttpAnonymousEvent,
    ) -> Result<(), ClientError> {
        self.call(RecordAnonymousEvent(stamped(event.clone())), None)
            .await
    }
}

/// `event` with [`ClientPlatform::CURRENT`] unless it already names a platform.
fn stamped(mut event: HttpAnonymousEvent) -> HttpAnonymousEvent {
    event.platform = event.platform.or(Some(ClientPlatform::CURRENT));
    event
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use zwipe_core::http::contracts::metrics::AnonymousEventKind;

    use super::*;

    #[test]
    fn an_unnamed_platform_is_stamped_and_a_named_one_kept() {
        let event = |platform| HttpAnonymousEvent {
            session_id: Uuid::nil(),
            kind: AnonymousEventKind::AppOpened,
            platform,
        };
        assert_eq!(stamped(event(None)).platform, Some(ClientPlatform::CURRENT));
        assert_eq!(
            stamped(event(Some(ClientPlatform::Terminal))).platform,
            Some(ClientPlatform::Terminal)
        );
    }
}
