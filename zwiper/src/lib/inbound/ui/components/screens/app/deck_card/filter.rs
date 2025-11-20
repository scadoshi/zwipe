pub mod mana;
pub mod printing;
pub mod stats;
pub mod text;
pub mod types;

use crate::inbound::ui::{
    components::{
        auth::bouncer::Bouncer,
        interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        screens::app::deck_card::filter::{
            mana::Mana, printing::Printing, stats::Stats, text::Text, types::Types,
        },
    },
    router::Router,
};
use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::SearchCards;

#[component]
pub fn Filter() -> Element {
    let filter: Signal<SearchCards> = use_context();

    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(|| SwipeState::new());
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
