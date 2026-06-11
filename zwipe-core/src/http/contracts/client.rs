//! Client metadata contracts (app version gating).

use serde::{Deserialize, Serialize};

/// Minimum app version the server supports.
///
/// Served publicly at `/api/client/min-version`. Clients below this version
/// render a blocking "Update required" screen. `"0.0.0"` means the gate is
/// open (every version allowed).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpMinClientVersion {
    /// Lowest app version allowed to talk to this server, e.g. `"1.0.5"`.
    pub min_version: String,
}
