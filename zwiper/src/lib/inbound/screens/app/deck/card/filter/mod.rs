pub mod combat;
pub mod mana;
pub mod rarity;
pub mod set;
pub mod text;
pub mod types;

use crate::inbound::{
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

                        div { class : "flex-row flex-between",
                            button { class : "btn btn-half",
                                onclick : move |_| {
                                    navigator.push(Router::Combat {});
                                },
                                "combat"
                            }

                            button { class : "btn btn-half",
                                onclick : move |_| {
                                    navigator.push(Router::Mana {});
                                },
                                "mana"
                            }
                        }

                        div { class : "flex-row flex-between",
                            button { class : "btn btn-half",
                                onclick : move |_| {
                                    navigator.push(Router::Rarity {});
                                },
                                "rarity"
                            }

                            button { class : "btn btn-half",
                                onclick : move |_| {
                                    navigator.push(Router::Set {});
                                },
                                "set"
                            }
                        }

                        div { class : "flex-row flex-between",
                            button { class : "btn btn-half text-center",
                                onclick : move |_| {
                                    navigator.push(Router::Text {});
                                },
                                "text"
                            }

                            button { class : "btn btn-half text-center",
                                onclick : move |_| {
                                    navigator.push(Router::Types {});
                                },
                                "types"
                            }
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
