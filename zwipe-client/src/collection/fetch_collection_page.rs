//! Fetch one page of the caller's collection.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::collection::{CollectionPageSize, CollectionSearch};
use zwipe_core::http::contracts::collection::HttpCollectionPage;
use zwipe_core::http::endpoints::collection::GetCollection;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the page of the signed-in user's collection from `offset`, `limit` rows long, of
    /// the rows whose card names hold `name` (every row when it is empty), sorted by card name.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_collection_page(
        &self,
        session: &Session,
        name: &CollectionSearch,
        offset: u32,
        limit: CollectionPageSize,
    ) -> Result<HttpCollectionPage, ClientError> {
        let endpoint = GetCollection {
            name: name.clone(),
            offset,
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
        let body = r#"{"cards":[],"total":120,"next_offset":50}"#.to_owned();
        let (client, server) = serve_once("200 OK", body).await;
        let search = CollectionSearch::new("sol ring").unwrap();

        let page = client
            .fetch_collection_page(&session(), &search, 0, CollectionPageSize::DEFAULT)
            .await
            .unwrap();

        assert_eq!(page.total, 120);
        assert_eq!(page.next_offset, Some(50));
        let request = server.await.unwrap();
        assert!(
            request.starts_with("GET /api/collection?name=sol%20ring&offset=0&limit=50 "),
            "{request}"
        );
        assert!(has_token(&request), "{request}");
    }

    #[tokio::test]
    async fn an_error_status_is_the_client_s() {
        let (client, _server) = serve_once("422 Unprocessable Entity", "\"bad\"".to_owned()).await;
        let error = client
            .fetch_collection_page(
                &session(),
                &CollectionSearch::default(),
                0,
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
