use keyring::default;
use zwipe::domain::auth::models::session::Session;

fn credential_username() -> String {
    env!("CARGO_PKG_NAME").to_string() + "-user"
}

fn credential_service() -> String {
    env!("CARGO_PKG_NAME").to_string() + "-service"
}

pub trait Persist {
    fn save(&self) -> anyhow::Result<()>;
    fn load() -> anyhow::Result<Option<Session>>;
    fn delete(&self) -> anyhow::Result<()>;
}

impl Persist for Session {
    fn save(&self) -> anyhow::Result<()> {
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        let bytes = serde_json::to_vec(self)?;
        credential.set_secret(&bytes)?;
        Ok(())
    }

    fn load() -> anyhow::Result<Option<Self>> {
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        let result = credential.get_secret();
        match result {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)?;
                if session.is_expired() {
                    credential.delete_credential()?;
                    return Ok(None);
                }
                Ok(Some(session))
            }
        }
    }

    fn delete(&self) -> anyhow::Result<()> {
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        credential.delete_credential()?;
        Ok(())
    }
}
