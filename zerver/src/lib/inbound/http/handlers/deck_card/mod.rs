//! Deck card composition handlers.

pub mod create_deck_card;
pub mod delete_deck_card;
pub mod get_deck_card;
pub mod import_deck_cards;
pub mod patch_deck_card;

pub use create_deck_card::create_deck_card;
pub use delete_deck_card::delete_deck_card;
pub use import_deck_cards::import_deck_cards;
pub use patch_deck_card::patch_deck_card;
use zwipe_core::domain::deck::Board;

use crate::inbound::http::ApiError;

/// Parses an optional board from a request body. An unknown value answers 422 with a fixed
/// message; the raw value is logged at debug and never echoed to the client.
///
/// # Errors
/// Returns [`ApiError::UnprocessableEntity`] when `raw` names no board.
pub(crate) fn parse_board(raw: Option<&str>) -> Result<Option<Board>, ApiError> {
    raw.map(|value| {
        Board::try_from(value).map_err(|_invalid| {
            tracing::debug!(board = value, "rejected an unknown board value");
            ApiError::UnprocessableEntity("invalid board value".to_string())
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_board_answers_an_unknown_value_with_the_fixed_message() {
        match parse_board(Some("graveyard")) {
            Err(ApiError::UnprocessableEntity(message)) => {
                assert_eq!(
                    message, "invalid board value",
                    "the raw value stays off the wire"
                );
            }
            other => panic!("expected a 422, got {other:?}"),
        }
    }

    #[test]
    fn parse_board_passes_known_and_absent_values() {
        assert!(matches!(
            parse_board(Some("sideboard")),
            Ok(Some(Board::Sideboard))
        ));
        assert!(matches!(parse_board(None), Ok(None)));
    }
}
