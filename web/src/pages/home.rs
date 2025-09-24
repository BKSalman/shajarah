use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

use crate::modules::tree::{components::tree::Tree, types::Position};

const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 5.0;
const PINCH_SENSITIVITY: f64 = 0.01;
const ZOOM_SENSITIVITY: f64 = 0.01;

#[component]
pub fn Home() -> Element {
    let mut offset = use_signal(|| Position { x: 0., y: 0. });
    let mut last_touch = use_signal(|| None::<Position>);

    let mut pointer_event_cache = use_store(|| Vec::<Event<PointerData>>::new());

    let mut prev_distance = use_signal(|| None::<f64>);
    let mut zoom = use_signal(|| 1.);

    let pointer_up_handler = move |pointer_event: Event<PointerData>| {
        pointer_event.prevent_default();
        pointer_event_cache.retain(|event| pointer_event.pointer_id() != event.pointer_id());

        if pointer_event_cache.len() < 2 {
            prev_distance.set(None);
        }
        last_touch.set(None);
    };

    let calculate_distance = |p1: &Position, p2: &Position| -> f64 {
        let dx = p1.x - p2.x;
        let dy = p1.y - p2.y;
        f64::sqrt(dx * dx + dy * dy)
    };

    let mut apply_zoom = move |delta: f64| {
        let old_zoom = zoom();
        let new_zoom = (old_zoom * (1.0 + delta)).clamp(MIN_ZOOM, MAX_ZOOM);
        zoom.set(new_zoom);
    };

    rsx! {
        div {
            width: "100vw",
            height: "100vh",
            overflow: "hidden",
            position: "relative",
            onmounted: move |evt| async move {
                if let Ok(r) = evt.data().get_client_rect().await {
                    offset.set(Position { x: r.width() / 2., y: r.height() / 2.});
                }
            },
            onpointerdown: move |pointer_down_event| {
                pointer_down_event.prevent_default();
                pointer_event_cache.push(pointer_down_event);

                if pointer_event_cache.len() == 2 {
                    last_touch.set(None);
                }
            },
            onpointerup: pointer_up_handler,
            onpointercancel: pointer_up_handler,
            onpointerleave: pointer_up_handler,
            onpointerout: pointer_up_handler,
            onpointermove: move |pointer_event| {
                pointer_event.prevent_default();
                {
                    let mut cache = pointer_event_cache.write();
                    if let Some(entry) = cache.iter_mut().find(|e| e.pointer_id() == pointer_event.pointer_id()) {
                        *entry = pointer_event.clone();
                    }
                }

                let cache = pointer_event_cache.read();

                if cache.len() == 2 {
                    let p1 = Position { x: cache[0].client_coordinates().x, y: cache[0].client_coordinates().y };
                    let p2 = Position { x: cache[1].client_coordinates().x, y: cache[1].client_coordinates().y };

                    let current_distance = calculate_distance(&p1, &p2);

                    if let Some(prev_dist) = prev_distance() {
                        let scale_delta = (current_distance / prev_dist - 1.0) * PINCH_SENSITIVITY * 100.0;
                        apply_zoom(scale_delta);
                    }

                    prev_distance.set(Some(current_distance));
                }

                if !cache.is_empty() {
                    let coords = cache[0].client_coordinates();
                    if let Some(last_touch) = last_touch() {
                        offset.with_mut(|offset| {
                            offset.x -= (last_touch.x - coords.x) * 0.9;
                            offset.y -= (last_touch.y - coords.y) * 0.9;
                        });
                    }

                    last_touch.set(Some(Position { x: coords.x, y: coords.y}));
                }
            },
            onwheel: move |wheel_event| {
                wheel_event.prevent_default();
                match wheel_event.delta() {
                    WheelDelta::Pixels(vector3_d) => {
                        // CONTROL is sent on the OS level when pinching on touchpad
                        if wheel_event.modifiers().contains(Modifiers::CONTROL) {
                            let delta = -vector3_d.y * ZOOM_SENSITIVITY * 2.0;
                            apply_zoom(delta);
                        } else {
                            offset.with_mut(|offset| {
                                offset.x -= vector3_d.x;
                                offset.y -= vector3_d.y;
                            });
                        }
                    },
                    _ => {},
                }
            },
            ontouchmove: move |move_event| {
                move_event.prevent_default();
            },
            ontouchstart: move |touch_event| {
                // touch_event.prevent_default();
            },
            ontouchend: move |touch_event| {
                // touch_event.prevent_default();
            },
            div {
                position: "absolute",
                left: "{offset().x}px",
                top: "{offset().y}px",
                transform: "translate(-50%, -50%) scale({zoom()})",
                display: "flex",
                Tree {}
            }
        }
    }
}
