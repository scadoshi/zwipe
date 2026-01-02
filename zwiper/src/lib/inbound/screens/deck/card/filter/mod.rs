pub mod combat;
pub mod mana;
pub mod rarity;
pub mod set;
pub mod text;
pub mod types;

use crate::inbound::{components::auth::bouncer::Bouncer, router::Router};
use dioxus::prelude::*;

#[component]
pub fn Filter() -> Element {
    let navigator = use_navigator();

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "container-sm",

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

                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
            }
        }
    }
}
