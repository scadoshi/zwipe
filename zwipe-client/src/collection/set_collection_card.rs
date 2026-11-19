//! Give one row of the caller's collection its copies.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::{HttpSetCollectionCard, HttpSetCollectionCardReply};
use zwipe_core::http::endpoints::collection::SetCollectionCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Gives the row `set` names (printing, finish, condition and language) its copies, 0 to
    /// remove it, and answers with the row as it is now, `None` once removed.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status (422 for a printing the collection doesn't take or a value it doesn't know,
    /// 429 past the rate limit), or the reply doesn't decode.
    pub async fn set_collection_card(
        &self,
        session: &Session,
        set: &HttpSetCollectionCard,
    ) -> Result<HttpSetCollectionCardReply, ClientError> {
        self.call(SetCollectionCard(set.clone()), Some(session))
            .await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use uuid::Uuid;
    use zwipe_core::domain::collection::{CardCondition, CardLanguage, Finish, SetQuantity};

    use super::super::test_server::{has_token, serve_once, session};
    use super::*;

    #[tokio::test]
    async fn it_puts_the_row_s_key_and_copies_and_reads_a_removal() {
        let (client, server) = serve_once("200 OK", r#"{"card":null}"#.to_owned()).await;
        let set = HttpSetCollectionCard {
            scryfall_data_id: Uuid::from_u128(7),
            finish: Finish::Foil,
            condition: Some(CardCondition::NearMint),
            language: Some(CardLanguage::Ja),
            quantity: SetQuantity::new(0).unwrap(),
        };

        let reply = client.set_collection_card(&session(), &set).await.unwrap();

        assert_eq!(reply.card, None, "0 removed the row");
        let request = server.await.unwrap();
        assert!(
            request.starts_with("PUT /api/collection/cards "),
            "{request}"
        );
        assert!(has_token(&request), "{request}");
        let (_, body) = request.split_once("\r\n\r\n").unwrap();
        let on_wire: HttpSetCollectionCard = serde_json::from_str(body).unwrap();
        assert_eq!(on_wire, set, "the whole key and the copies go");
    }

    #[tokio::test]
    async fn a_refusal_is_the_client_s() {
        let (client, _server) = serve_once("429 Too Many Requests", "\"slow\"".to_owned()).await;
        let set = HttpSetCollectionCard {
            scryfall_data_id: Uuid::from_u128(7),
            finish: Finish::Nonfoil,
            condition: None,
            language: None,
            quantity: SetQuantity::new(2).unwrap(),
        };
        let error = client
            .set_collection_card(&session(), &set)
            .await
            .unwrap_err();
        assert!(
            matches!(error, ClientError::TooManyRequests(_)),
            "{error:?}"
        );
    }
}
