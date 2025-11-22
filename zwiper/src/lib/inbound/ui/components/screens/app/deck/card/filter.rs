pub mod combat;
pub mod mana;
pub mod printing;
pub mod text;
pub mod types;

use crate::inbound::ui::{
    components::{
        auth::bouncer::Bouncer,
        interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
    },
    router::Router,
};
use dioxus::prelude::*;

#[component]
pub fn Filter() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(SwipeState::new);
    let navigator = use_navigator();

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "card filters" }

                    form { class : "flex-col text-center",

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::Types {});
                            },
                            "types"
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::Text {});
                            },
                            "text"
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::Mana {});
                            },
                            "mana"
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::Stats {});
                            },
                            "stats"
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::Printing {});
                            },
                            "printing"
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                navigator.go_back();
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}
