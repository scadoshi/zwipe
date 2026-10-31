//! The session above the router: its refresh, its upkeep loop, the gate on
//! authed routes, and logout.

pub mod auth_gate;
pub mod ensure_fresh;
pub mod funnel;
pub mod logout;
pub mod upkeep;
