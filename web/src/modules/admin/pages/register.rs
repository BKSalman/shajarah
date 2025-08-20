use crate::{
    Route,
    modules::admin::{server::register_admin, types::RegisterInput},
};
use dioxus::prelude::*;
use garde::Validate as _;

#[component]
pub fn AdminRegister() -> Element {
    let mut error = use_signal(|| None::<String>);
    let mut first_name = use_signal(|| String::new());
    let mut last_name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    rsx! {
        if let Some(error) = error() {
            div { class: "text-red-500", "{error}" }
        }
        form {
            autocomplete: "off",
            onsubmit: move |e| {
                e.prevent_default();
                async move {
                    let register_input = RegisterInput {
                        first_name: first_name(),
                        last_name: last_name(),
                        email: email(),
                        password: password(),
                    };
                    if let Err(e) = register_input.validate() {
                        error.set(Some(e.to_string()));
                        return;
                    }
                    match register_admin(register_input).await {
                        Ok(_) => {
                            tracing::info!("redirecting");
                            navigator().replace(Route::AdminLogin);
                        }
                        Err(e) => {
                            tracing::error!("{e}");
                        }
                    }
                }
            },
            div {
                input {
                    class: "input",
                    placeholder: "الاسم",
                    oninput: move |e| {
                        error.set(None);
                        first_name.set(e.value())
                    },
                }
                input {
                    class: "input",
                    placeholder: "الاسم الاخير",
                    oninput: move |e| {
                        error.set(None);
                        last_name.set(e.value())
                    },
                }
                input {
                    class: "input",
                    r#type: "email",
                    placeholder: "البريد",
                    oninput: move |e| {
                        error.set(None);
                        email.set(e.value());
                    },
                }
                input {
                    class: "input",
                    r#type: "password",
                    placeholder: "كلمة المرور",
                    oninput: move |e| {
                        error.set(None);
                        password.set(e.value());
                    },
                }
            }
            button { class: "btn btn-primary", "التسجيل" }
        }
    }
}
