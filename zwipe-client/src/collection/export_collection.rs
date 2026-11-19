//! Fetch the caller's collection as a ManaBox export.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::HttpCollectionExport;
use zwipe_core::http::endpoints::collection::ExportCollection;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the signed-in user's collection as a ManaBox export: a file name and the file's
    /// text, which a collection import reads back.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status (429 past the rate limit it shares with the import), or the reply doesn't
    /// decode.
    pub async fn export_collection(
        &self,
        session: &Session,
    ) -> Result<HttpCollectionExport, ClientError> {
        self.call(ExportCollection, Some(session)).await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::super::test_server::{has_token, serve_once, session};

    #[tokio::test]
    async fn it_asks_for_the_export_with_the_session_and_reads_the_file() {
        let body = r#"{"file_name":"zwipe-collection.csv","csv":"Name,Set code\nSol Ring,c21\n"}"#;
        let (client, server) = serve_once("200 OK", body.to_owned()).await;

        let export = client.export_collection(&session()).await.unwrap();

        assert_eq!(export.file_name, "zwipe-collection.csv");
        assert!(export.csv.starts_with("Name,Set code\n"));
        let request = server.await.unwrap();
        assert!(
            request.starts_with("GET /api/collection/export "),
            "{request}"
        );
        assert!(has_token(&request), "{request}");
    }
}
