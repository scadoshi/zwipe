//! The one place an API request is built, sent and decoded.
//!
//! Every endpoint goes through [`ZwipeClient::call`]; the per-endpoint files
//! describe the call and hold no transport code.

use crate::{ClientError, ZwipeClient};
use tracing::info;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::endpoint::{Endpoint, Method},
};

impl ZwipeClient {
    /// Sends `endpoint` and decodes its success body.
    ///
    /// `session` is required when `E::AUTH`; passing `None` there is a bug in
    /// the caller, so it fails as an unauthorized error rather than sending an
    /// anonymous request the server would reject anyway.
    pub async fn call<E: Endpoint>(
        &self,
        endpoint: E,
        session: Option<&Session>,
    ) -> Result<E::Response, ClientError> {
        let mut url = self.base_url.clone();
        url.set_path(&endpoint.path());

        let mut request = match E::METHOD {
            Method::Get => self.client.get(url.clone()),
            Method::Post => self.client.post(url.clone()),
            Method::Put => self.client.put(url.clone()),
            Method::Patch => self.client.patch(url.clone()),
            Method::Delete => self.client.delete(url.clone()),
        };

        if E::AUTH {
            let Some(session) = session else {
                return Err(ClientError::Unauthorized("not logged in".to_string()));
            };
            request = request.bearer_auth(&*session.access_token.value);
        }

        if let Some(body) = endpoint.body() {
            request = request.json(body);
        }

        info!("{} {}", E::METHOD.as_str(), url);
        decode::<E>(request.send().await?).await
    }
}

/// Splits the response into success and failure, and turns a success body into
/// `E::Response`. Any 2xx is success: several endpoints answer 200 or 204 for
/// the same call, and no caller branches on which. An empty body reads as
/// `null` so those can declare `type Response = ()`.
async fn decode<E: Endpoint>(response: reqwest::Response) -> Result<E::Response, ClientError> {
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await?;
        return Err((status, message).into());
    }

    let bytes = response.bytes().await?;
    let decoded = if bytes.is_empty() {
        serde_json::from_str::<E::Response>("null")?
    } else {
        serde_json::from_slice::<E::Response>(&bytes)?
    };
    Ok(decoded)
}
