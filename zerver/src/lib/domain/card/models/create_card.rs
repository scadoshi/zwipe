#[cfg(feature = "zerver")]
use crate::domain::card::models::scryfall_data::get_scryfall_data::GetScryfallDataError;
#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("scryfall data inserted but database returned invalid object: {0}")]
    ScryfallDataFromDb(anyhow::Error),
    #[error("card profile created but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
    #[error(transparent)]
    GetScryfallData(anyhow::Error),
}

#[cfg(feature = "zerver")]
impl From<GetScryfallDataError> for CreateCardError {
    fn from(value: GetScryfallDataError) -> Self {
        Self::GetScryfallData(value.into())
    }
}
