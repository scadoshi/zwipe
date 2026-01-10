use serde_json::Value;
use zwipe::{
    config::Config,
    domain::card::models::scryfall_data::ScryfallData,
    inbound::external::scryfall::bulk::{BulkEndpoint, CACHE_BULK_PATH},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(config.rust_log)
        .init();
    let full_cache_path =
        CACHE_BULK_PATH.to_string() + BulkEndpoint::AllCards.to_snake_case().as_str() + ".json";
    let cache_read_result = std::fs::read(&full_cache_path);
    match cache_read_result {
        Ok(bytes) => {
            tracing::info!("successfully read cache file");
            tracing::info!("printing bytes for debug");

            let some_bytes = bytes
                .iter()
                .take(1_000)
                .map(|byte| char::from(*byte))
                .collect::<String>();
            tracing::info!("{}", some_bytes);

            let cards_json_result = serde_json::from_slice::<Value>(&bytes);
            match cards_json_result {
                Ok(cards_json) => {
                    tracing::info!("successfully parse card json");
                    let cards_result = serde_json::from_value::<Vec<ScryfallData>>(cards_json);
                    match cards_result {
                        Ok(_cards) => {
                            tracing::info!("successfully parse cards");
                            return Ok(());
                        }
                        Err(e) => {
                            tracing::warn!("failed to parse `Vec<ScryfallData>`: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to parse json from cache file: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("failed to read cache file: {}", e);
        }
    }
    tracing::info!("successfully parsed cache into scryfall data");
    Ok(())
}
