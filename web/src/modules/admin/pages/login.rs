use std::collections::HashMap;

use crate::{
    Route,
    modules::admin::{server::login_admin, types::LoginData},
};
use dioxus::prelude::*;
use garde::Validate;

#[component]
pub fn AdminLogin() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut remember = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut field_errors = use_signal(|| HashMap::<String, String>::new());

    let is_form_valid = move || -> bool {
        let login_data = LoginData {
            email: email.read().trim().to_lowercase(),
            password: password.read().clone(),
        };

        login_data.validate().is_ok()
    };

    let mut clear_field_error = move |field: &str| {
        field_errors.with_mut(|errors| {
            errors.remove(field);
        });
    };

    let get_field_error =
        move |field: &str| -> Option<String> { field_errors.read().get(field).cloned() };

    let has_field_error = move |field: &str| -> bool { field_errors.read().contains_key(field) };

    rsx! {
        div {
            dir: "rtl",
            class: "min-h-screen flex items-center justify-center bg-tree-texture p-6",

            div {
                class: "card card-forest w-full max-w-md fade-in hover:shadow-forest",

                div {
                    class: "card-body",

                    // Header
                    div {
                        class: "text-center mb-6",
                        h1 {
                            class: "text-3xl font-bold text-forest-dark mb-2",
                            "شجرة"
                        }
                        h2 {
                            class: "text-xl font-semibold text-forest-primary",
                            "تسجيل الدخول"
                        }
                        p {
                            class: "text-sm text-gray-500 mt-2",
                            "ادخل بياناتك للوصول إلى حسابك"
                        }
                    }

                    // Form
                    form {
                        class: "space-y-4",
                        onsubmit: move |evt: FormEvent| {
                            evt.prevent_default();
                            async move {
                                if is_submitting() {
                                    return;
                                }

                                // Clear previous errors
                                error_message.set(None);
                                field_errors.set(HashMap::new());

                                is_submitting.set(true);

                                let login_data = LoginData {
                                    email: email.read().trim().to_lowercase(),
                                    password: password.read().clone(),
                                };

                                if let Err(report) = login_data.validate() {
                                    for (field_name, field_report) in report.iter() {
                                        let field_name = field_name.to_string();
                                        match &field_name.to_string()[..] {
                                            "email" => {
                                                let mut errors = field_errors.write();
                                                errors.insert(field_name, field_report.message().to_string());
                                            }
                                            "password" => {
                                                let mut errors = field_errors.write();
                                                errors.insert(field_name, field_report.message().to_string());
                                            }
                                            e => {
                                                tracing::error!("validation error: {e}");
                                            }
                                        }
                                    }

                                    return;
                                }

                                match login_admin(login_data).await {
                                    Ok(_) => {
                                        navigator().replace(Route::Admin);
                                    }
                                    Err(server_error) => {
                                        error_message.set(Some(server_error.to_string()));
                                    }
                                }
                                is_submitting.set(false);
                            }
                        },

                        // Email Field
                        div {
                            class: "form-group",
                            label {
                                r#for: "email",
                                class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207"
                                    }
                                }
                                "البريد الإلكتروني"
                            }
                            input {
                                id: "email",
                                r#type: "email",
                                required: true,
                                disabled: is_submitting(),
                                class: "input w-full",
                                class: if has_field_error("email") {
                                    "border-red-300 focus:border-red-500 focus:shadow-red"
                                },
                                class: if is_submitting() { "loading" },
                                placeholder: "ادخل بريدك الإلكتروني",
                                autocomplete: "email",
                                dir: "rtl",
                                value: "{email}",
                                oninput: move |evt| {
                                    email.set(evt.value());
                                    clear_field_error("email");
                                }
                            }
                            if let Some(error) = get_field_error("email") {
                                p {
                                    class: "text-sm text-red-600 mt-1",
                                    "{error}"
                                }
                            }
                        }

                        // Password Field
                        div {
                            class: "form-group",
                            label {
                                r#for: "password",
                                class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                                    }
                                }
                                "كلمة المرور"
                            }
                            div {
                                class: "relative",
                                input {
                                    id: "password",
                                    r#type: if *show_password.read() { "text" } else { "password" },
                                    required: true,
                                    disabled: is_submitting(),
                                    class: "input w-full pr-10",
                                    class: if has_field_error("password") {
                                        "border-red-300 focus:border-red-500"
                                    },
                                    class: if is_submitting() { "loading" },
                                    placeholder: "ادخل كلمة المرور",
                                    autocomplete: "current-password",
                                    dir: "rtl",
                                    value: "{password}",
                                    oninput: move |evt| {
                                        password.set(evt.value());
                                        clear_field_error("password");
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600",
                                    tabindex: "-1",
                                    onclick: move |_| show_password.set(!show_password()),

                                    if !*show_password.read() {
                                        // Eye open icon
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                            }
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                                            }
                                        }
                                    } else {
                                        // Eye closed icon
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21"
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(error) = get_field_error("password") {
                                p {
                                    class: "text-sm text-red-600 mt-1",
                                    "{error}"
                                }
                            }
                        }

                        // Remember Me and Forgot Password
                        div {
                            class: "flex items-center justify-between",
                            label {
                                class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "rounded border-gray-300 text-primary-600 focus:ring-primary-500",
                                    checked: *remember.read(),
                                    onchange: move |evt| remember.set(evt.checked())
                                }
                                span {
                                    class: "mr-2 text-sm text-gray-600",
                                    "تذكرني"
                                }
                            }
                            a {
                                href: "#",
                                class: "text-sm text-primary-600 hover:text-primary-500",
                                "نسيت كلمة المرور؟"
                            }
                        }

                        // Submit Button
                        button {
                            r#type: "submit",
                            disabled: is_submitting() || !is_form_valid(),
                            class: "btn btn-primary w-full btn-lg",
                            class: if is_submitting() || !is_form_valid() { "opacity-50 cursor-not-allowed loading" },

                            if is_submitting() {
                                svg {
                                    class: "animate-spin -ml-1 mr-3 h-4 w-4 text-white",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    circle {
                                        class: "opacity-25",
                                        cx: "12",
                                        cy: "12",
                                        r: "10",
                                        stroke: "currentColor",
                                        stroke_width: "4"
                                    }
                                    path {
                                        class: "opacity-75",
                                        fill: "currentColor",
                                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                    }
                                }
                                "جاري تسجيل الدخول..."
                            } else {
                                "تسجيل الدخول"
                            }
                        }
                    }

                    // Error Alert
                    if let Some(error) = error_message.read().as_ref() {
                        div {
                            class: "alert text-red-500 alert-error mt-4",
                            div {
                                class: "flex items-center",
                                svg {
                                    class: "w-5 h-5 ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                    }
                                }
                                p { "{error}" }
                            }
                        }
                    }

                    // Register Link
                    div {
                        class: "text-center mt-6 pt-6 border-t border-forest-light",
                        p {
                            class: "text-sm text-gray-600",
                            "لا تملك حساباً؟ "
                            Link {
                                to: Route::AdminRegister,
                                class: "text-primary-600 hover:text-primary-500 font-medium",
                                "أنشئ حساب جديد"
                            }
                        }
                    }
                }
            }
        }
    }
}
