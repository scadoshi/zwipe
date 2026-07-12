#[cfg(feature = "zerver")]
use axum::Json;
#[cfg(feature = "zerver")]
use reqwest::StatusCode;
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::mechanical_category::{CardRole, CardRoleView};

/// Returns the full card-role catalog (slug, display name, short name), built
/// straight from the `CardRole` enum — no DB read. The server-delivered role
/// catalog that lets a client label any role slug without a compiled enum, so
/// new roles reach clients on deploy (see server_driven_catalogs.md, Part B).
#[cfg(feature = "zerver")]
pub async fn get_card_roles() -> (StatusCode, Json<Vec<CardRoleView>>) {
    let roles: Vec<CardRoleView> = CardRole::all().iter().map(CardRole::to_view).collect();
    (StatusCode::OK, Json(roles))
}
