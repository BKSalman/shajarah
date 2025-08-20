use crate::{
    Route,
    modules::admin::{server::login_admin, types::LoginInput},
};
use dioxus::prelude::*;
use garde::Validate;
#[component]
pub fn AdminLogin() -> Element {
    let mut error = use_signal(|| None::<String>);
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    rsx! {
        form {
            autocomplete: "off",
            onsubmit: move |e| {
                e.prevent_default();
                async move {
                    let login_input = LoginInput {
                        email: email(),
                        password: password(),
                    };
                    if let Err(e) = login_input.validate() {
                        error.set(Some(e.to_string()));
                        return;
                    }
                    match login_admin(login_input).await {
                        Ok(_) => {
                            tracing::info!("redirecting");
                            navigator().replace(Route::Admin);
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
            button { class: "btn btn-primary", "تسجيل الدخول" }
        }
    }
}
