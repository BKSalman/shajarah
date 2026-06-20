use std::collections::HashMap;

use crate::modules::user::types::{RegisterData, RegisterDataStoreExt};
use crate::{Route, i18n::Arabic, modules::admin::server::register_admin};
use dioxus::prelude::*;
use garde::Validate as _;

#[component]
pub fn AdminRegister() -> Element {
    let register_data = use_store(|| RegisterData {
        first_name: None,
        last_name: None,
        email: None,
        password: None,
        confirm_password: None,
    });

    let mut field_errors = use_signal(HashMap::<String, String>::new);

    let mut error_message = use_signal(|| Option::<String>::None);

    let get_field_error =
        move |field: &str| -> Option<String> { field_errors.read().get(field).cloned() };

    let mut clear_field_error = move |field: &str| {
        field_errors.with_mut(|errors| {
            errors.remove(field);
        });
    };

    let mut clear_errors = move || {
        field_errors.write().clear();
        error_message.write().take();
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-tree-texture p-6",

            div { class: "card card-forest w-full max-w-md fade-in hover:shadow-forest",

                div { class: "card-body",

                    // Header
                    div { class: "text-center mb-6",
                        h1 { class: "text-3xl font-bold text-forest-dark mb-2", "شجرة" }
                        h2 { class: "text-xl font-semibold text-forest-primary",
                            "تسجيل حساب جديد"
                        }
                        p { class: "text-sm text-gray-500 mt-2",
                            "أنشئ حساباً جديداً للانضمام إلى شجرة العائلة"
                        }
                    }

                    // Form
                    form {
                        class: "space-y-4",
                        autocomplete: "off",
                        onsubmit: move |e| {
                            e.prevent_default();
                            async move {
                                let register_data = register_data();

                                clear_errors();

                                if let Err(report) = garde::with_i18n(Arabic, || register_data.validate()) {
                                    for (field_name, field_report) in report.iter() {
                                        let mut errors = field_errors.write();
                                        errors

                                            .insert(
                                                field_name.to_string(),
                                                field_report.message().to_string(),
                                            );
                                    }
                                    return;
                                }
                                match register_admin(register_data).await {
                                    Ok(_) => {
                                        navigator().replace(Route::AdminLogin);
                                    }
                                    Err(e) => {
                                        tracing::error!("{e:?}");
                                    }
                                }
                            }
                        },

                        // First and Last Name
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                            div { class: "form-group min-w-0",
                                label { r#for: "first_name", class: "form-label",
                                    "الاسم الأول"
                                }
                                input {
                                    id: "first_name",
                                    r#type: "text",
                                    required: true,
                                    class: "input w-full",
                                    placeholder: "ادخل اسمك الأول",
                                    dir: "auto",
                                    oninput: move |evt| {
                                        clear_field_error("first_name");
                                        register_data.first_name().set(Some(evt.value()))
                                    },
                                }
                            }
                            div { class: "form-group min-w-0",
                                label { r#for: "last_name", class: "form-label",
                                    "الاسم الأخير"
                                }
                                input {
                                    id: "last_name",
                                    r#type: "text",
                                    required: true,
                                    class: "input w-full",
                                    placeholder: "ادخل اسمك الأخير",
                                    dir: "auto",
                                    oninput: move |evt| {
                                        clear_field_error("last_name");
                                        register_data.last_name().set(Some(evt.value()))
                                    },
                                }
                            }
                            if let Some(error) = get_field_error("first_name") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                            if let Some(error) = get_field_error("last_name") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Email
                        div { class: "form-group",
                            label { r#for: "email", class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-1",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207",
                                    }
                                }
                                "البريد الإلكتروني"
                            }
                            input {
                                id: "email",
                                r#type: "email",
                                required: true,
                                class: "input w-full",
                                placeholder: "ادخل بريدك الإلكتروني",
                                autocomplete: "email",
                                dir: "auto",
                                oninput: move |evt| {
                                    clear_field_error("email");
                                    register_data.email().set(Some(evt.value()))
                                },
                            }
                            if let Some(error) = get_field_error("email") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Password
                        div { class: "form-group",
                            label { r#for: "password", class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-1",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
                                    }
                                }
                                "كلمة المرور"
                            }
                            input {
                                id: "password",
                                r#type: "password",
                                required: true,
                                class: "input w-full",
                                placeholder: "اختر كلمة مرور قوية",
                                autocomplete: "new-password",
                                dir: "auto",
                                oninput: move |evt| {
                                    clear_field_error("password");
                                    register_data.password().set(Some(evt.value()))
                                },
                            }
                            if let Some(error) = get_field_error("password") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Confirm Password
                        div { class: "form-group",
                            label {
                                r#for: "confirm_password",
                                class: "form-label",
                                "تأكيد كلمة المرور"
                            }
                            input {
                                id: "confirm_password",
                                r#type: "password",
                                required: true,
                                class: "input w-full",
                                placeholder: "أعد إدخال كلمة المرور",
                                autocomplete: "new-password",
                                dir: "auto",
                                oninput: move |evt| {
                                    clear_field_error("confirm_password");
                                    register_data.confirm_password().set(Some(evt.value()))
                                },
                            }
                            if let Some(error) = get_field_error("confirm_password") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Submit Button
                        button {
                            r#type: "submit",
                            class: "btn btn-success w-full btn-lg",
                            "إنشاء الحساب"
                        }
                    }

                    // Error Alert
                    if let Some(error) = error_message.read().as_ref() {
                        div { class: "alert text-red-500 alert-error mt-4",
                            div { class: "flex items-center",
                                svg {
                                    class: "w-5 h-5 ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                                    }
                                }
                                p { "{error}" }
                            }
                        }
                    }

                    // Login Link
                    div { class: "text-center mt-6 pt-6 border-t border-gray-200",
                        p { class: "text-sm text-gray-600",
                            "هل لديك حساب بالفعل؟ "
                            Link {
                                to: Route::AdminLogin {},
                                class: "text-primary-600 hover:text-primary-500 font-medium",
                                "تسجيل الدخول"
                            }
                        }
                    }
                }
            }
        }
    }
}
