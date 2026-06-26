use dioxus::prelude::*;

/// A single option in the combobox list
#[derive(Debug, Clone, PartialEq)]
pub struct ComboboxOption<T: core::fmt::Debug + Clone + PartialEq> {
    pub index: usize,
    pub value: T,
    pub label: String, // display text (e.g. full_name)
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: core::fmt::Debug + Clone + PartialEq + 'static> {
    /// Called when the user types — use this to update `options` with search results
    pub on_search: EventHandler<String>,
    /// Called when the user selects an option
    pub on_select: EventHandler<ComboboxOption<T>>,
    /// The current list of options to display
    pub options: Vec<ComboboxOption<T>>,
    #[props(default = "Search...".to_string())]
    pub placeholder: String,
    #[props(default = None)]
    pub on_mount: Option<EventHandler<Event<MountedData>>>,
}

#[component]
pub fn Combobox<T: core::fmt::Debug + Clone + PartialEq + 'static>(
    props: ComboboxProps<T>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut highlighted: Signal<Option<usize>> = use_signal(|| None);
    let mut open = use_signal(|| false);

    let is_open = open() && !props.options.is_empty();

    let on_input = {
        let on_search = props.on_search;
        move |e: Event<FormData>| {
            let val = e.value();
            query.set(val.clone());
            highlighted.set(None);
            open.set(true);
            on_search.call(val);
        }
    };

    let on_keydown = {
        let options = props.options.clone();
        let on_select = props.on_select;
        move |e: Event<KeyboardData>| {
            let len = options.len();
            if len == 0 {
                return;
            }
            match e.key() {
                Key::ArrowDown => {
                    e.prevent_default();
                    highlighted.set(Some(
                        highlighted().map(|i| (i + 1).min(len - 1)).unwrap_or(0),
                    ));
                }
                Key::ArrowUp => {
                    e.prevent_default();
                    highlighted
                        .set(highlighted().and_then(|i| if i == 0 { None } else { Some(i - 1) }));
                }
                Key::Enter => {
                    if let Some(idx) = highlighted() {
                        let opt = options[idx].clone();
                        query.set(opt.label.clone());
                        on_select.call(opt);
                        open.set(false);
                        highlighted.set(None);
                    }
                }
                Key::Escape => {
                    open.set(false);
                    highlighted.set(None);
                }
                _ => {}
            }
        }
    };

    use_effect(move || {
        if let Some(idx) = highlighted() {
            let id = format!("combobox-option-{idx}");
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(&id))
            {
                let options = web_sys::ScrollIntoViewOptions::new();
                options.set_behavior(web_sys::ScrollBehavior::Instant);
                options.set_block(web_sys::ScrollLogicalPosition::Nearest);
                options.set_inline(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&options);
            }
        }
    });

    rsx! {
        div {
            class: "relative w-full",
            input {
                class: "input w-full",
                value: "{query}",
                placeholder: "{props.placeholder}",
                oninput: on_input,
                onkeydown: on_keydown,
                // close on blur, slight delay so onclick on option fires first
                onblur: move |_| {
                    spawn(async move {
                        gloo_timers::future::TimeoutFuture::new(150).await;
                        open.set(false);
                    });
                },
                onfocus: move |_| {
                    if !query.read().is_empty() {
                        open.set(true);
                    }
                },
                onmounted: move |element| {
                    if let Some(on_mount) = props.on_mount {
                        on_mount(element);
                    }
                },
            }

            if is_open {
                ul {
                    class: "absolute z-50 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                    for (idx, opt) in props.options.iter().enumerate() {
                        {
                            let opt = opt.clone();
                            let opt_click = opt.clone();
                            let on_select = props.on_select;
                            let is_highlighted = highlighted().map(|i| i == idx).unwrap_or(false);
                            rsx! {
                                li {
                                    key: "{opt.index}",
                                    id: "combobox-option-{idx}",
                                    class: if is_highlighted {
                                        "px-4 py-2 cursor-pointer bg-gray-100 text-gray-900"
                                    } else {
                                        "px-4 py-2 cursor-pointer hover:bg-gray-50 text-gray-700"
                                    },
                                    onmouseenter: move |_| highlighted.set(Some(idx)),
                                    onclick: move |_| {
                                        query.set(opt_click.label.clone());
                                        on_select.call(opt_click.clone());
                                        open.set(false);
                                        highlighted.set(None);
                                    },
                                    "{opt.label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
