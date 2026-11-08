//! The session above the router: its refresh, its upkeep loop, the gates on
//! the authed and signed-out routes, the cross-tab refresh lock, logout, and
//! where a signed-out visit returns after sign-in.

pub mod auth_gate;
pub mod cross_tab;
pub mod ensure_fresh;
pub mod funnel;
pub mod logout;
pub mod return_to;
pub mod settings_sync;
pub mod signed_out_gate;
pub mod upkeep;
