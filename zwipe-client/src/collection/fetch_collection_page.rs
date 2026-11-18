//! Fetch one page of the caller's collection.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::collection::{CollectionCursor, CollectionPageSize, CollectionSearch};
use zwipe_core::http::contracts::collection::HttpCollectionPage;
use zwipe_core::http::endpoints::collection::GetCollection;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the page of the signed-in user's collection after `after` (a reply's `next`; `None`
    /// for the first page), `limit` rows long, of the rows whose card names hold `name` (every
    /// row when it is empty), sorted by card name.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_collection_page(
        &self,
        session: &Session,
        name: &CollectionSearch,
        after: Option<&CollectionCursor>,
        limit: CollectionPageSize,
    ) -> Result<HttpCollectionPage, ClientError> {
        let endpoint = GetCollection {
            name: name.clone(),
            after: after.cloned(),
            limit,
        };
        self.call(endpoint, Some(session)).await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::super::test_server::{has_token, serve_once, session};
    use super::*;

    #[tokio::test]
    async fn it_asks_for_the_page_with_the_session_and_reads_it() {
        let body = r#"{"cards":[],"total":120,"next":"0a1b"}"#.to_owned();
        let (client, server) = serve_once("200 OK", body).await;
        let search = CollectionSearch::new("sol ring").unwrap();

        let page = client
            .fetch_collection_page(&session(), &search, None, CollectionPageSize::DEFAULT)
            .await
            .unwrap();

        assert_eq!(page.total, 120);
        let next = page.next.unwrap();
        assert_eq!(next.as_str(), "0a1b");
        let request = server.await.unwrap();
        assert!(
            request.starts_with("GET /api/collection?name=sol%20ring&limit=50 "),
            "the first page sends no cursor: {request}"
        );
        assert!(has_token(&request), "{request}");

        let (client, server) = serve_once(
            "200 OK",
            r#"{"cards":[],"total":120,"next":null}"#.to_owned(),
        )
        .await;
        let page = client
            .fetch_collection_page(
                &session(),
                &search,
                Some(&next),
                CollectionPageSize::DEFAULT,
            )
            .await
            .unwrap();
        assert_eq!(page.next, None);
        let request = server.await.unwrap();
        assert!(
            request.starts_with("GET /api/collection?name=sol%20ring&after=0a1b&limit=50 "),
            "the next page hands the cursor back: {request}"
        );
    }

    #[tokio::test]
    async fn an_error_status_is_the_client_s() {
        let (client, _server) = serve_once("422 Unprocessable Entity", "\"bad\"".to_owned()).await;
        let error = client
            .fetch_collection_page(
                &session(),
                &CollectionSearch::default(),
                None,
                CollectionPageSize::DEFAULT,
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, ClientError::UnprocessableEntity(_)),
            "{error:?}"
        );
    }
}
