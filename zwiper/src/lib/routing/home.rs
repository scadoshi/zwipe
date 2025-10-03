use dioxus::{html::input_data::MouseButton, prelude::*};
use zwipe::domain::ascii_logo;

use crate::{routing::Route, swipe};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let ascii_logo = ascii_logo::logo();
    let mut swipe_state = use_signal(|| swipe::State::new());

    rsx! {
        div { class : "swipe-able",

            ontouchstart : move |e: Event<TouchData>| {
                if let Some(t) = e.touches().into_iter().next() {
                    swipe_state.with_mut(|ss| {
                        ss.start = Some(t.client_coordinates());
                        ss.current = ss.start.clone();
                    });

                    println!("starting at {:?}", swipe_state.read().start);
                }

            },

            ontouchmove : move |e: Event<TouchData>| {
                if let Some(t) = e.touches().into_iter().next() {
                    swipe_state.with_mut(|ss| {
                        ss.moving = true;
                        ss.current = Some(t.client_coordinates());
                        if let Some(start) = ss.start {
                            if let Some(current) = ss.current {
                                ss.dx = start.x - current.x;
                                ss.dy = start.y - current.y;
                            }
                        }
                    });

                    let ss = swipe_state.read();
                    println!("moving at {:?}", ss.current);
                    println!("dx={:?}, dy={:?}", ss.dx, ss.dy);
                }
            },

            ontouchend : move |e: Event<TouchData>| {
                swipe_state.with_mut(|ss| {
                    ss.moving = false;
                    if let Some(t) = e.touches().into_iter().next() {
                        ss.current = Some(t.client_coordinates());
                    }
                });
                let ss = swipe_state.read();
                println!("ending at {:?}", ss.current);
            },

            onmousedown : move |e: Event<MouseData>| {
                    swipe_state.with_mut(|ss| {
                        ss.start = Some(e.client_coordinates());
                        ss.current = ss.start.clone();
                    });

                    println!("starting at {:?}", swipe_state.read().start);
            },

            onmousemove : move |e: Event<MouseData>| {
                if e.held_buttons().contains(MouseButton::Primary) {
                    swipe_state.with_mut(|ss| {
                    ss.moving = true;
                    ss.current = Some(e.client_coordinates());
                    if let Some(start) = ss.start {
                        if let Some(current) = ss.current {
                            ss.dx = start.x - current.x;
                            ss.dy = start.y - current.y;
                        }
                    }
                    });

                    let ss = swipe_state.read();
                    println!("moving at {:?}", ss.current);
                    println!("dx={:?}, dy={:?}", ss.dx, ss.dy);
                }
            },

            onmouseup : move |e: Event<MouseData>| {
                swipe_state.with_mut(|ss| {
                    ss.moving = false;
                    ss.current = Some(e.client_coordinates());
                });
                let ss = swipe_state.read();
                println!("ending at {:?}", ss.current);
            },

            div { class : "home-screen",
                div {
                    onclick : move |_| {
                        navigator.push(Route::Login {});
                    },
                    class : "home-direction-arrow",
                    "↑"
                },

                p { "swipe ", b { "up" }, " to ", b { "log in" } },
                br {}, br {},
                pre { class: "ascii-logo", "{ascii_logo}" },
                br {}, br {},
                p { "swipe ", b { "down" }, " to ", b { "create profile" } },

                div {
                    onclick : move |_| {
                        navigator.push(Route::Register {});
                    },
                    class : "home-direction-arrow",
                    "↓"
                }
            }
        }
    }
}
