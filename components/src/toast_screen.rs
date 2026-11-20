//! The screen the toast stack is told about: each route that lands moves it to the route's path,
//! so a toast about the screen being left leaves with it, and a carried one stays
//! ([`zwipe_ui::toast::Life`]).

use dioxus::prelude::*;
use dioxus::router::{GenericRouterContext, RouterConfig};

/// The router's config for a client whose toast stack watches `screen`: each route that lands
/// moves `screen` to its path, without the query ([`zwipe_ui::toast::screen`]).
pub fn moving_screen<R: Routable>(screen: Signal<String>) -> RouterConfig<R> {
    RouterConfig::default().on_update(move |at: GenericRouterContext<R>| {
        let path = at.current().to_string();
        let now = zwipe_ui::toast::screen(&path);
        // The callback is `Fn`, so it writes through its own copy of the signal.
        let mut screen = screen;
        if *screen.peek() != now {
            screen.set(now.to_owned());
        }
        None
    })
}
