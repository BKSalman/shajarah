use crate::modules::add_request::AddMember;
use crate::modules::admin::pages::{Admin, login::AdminLogin, register::AdminRegister};
use crate::pages::Home;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod middleware;
pub mod modules;
pub mod pages;
#[cfg(feature = "server")]
pub mod server;
pub mod ui;
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<Vec<String>>,
}
#[cfg(feature = "server")]
impl axum::response::IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self)
            .expect("ErrorResponse as json")
            .into_response()
    }
}
#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Home,
    #[route("/admin")]
    Admin,
    #[route("/admin/login")]
    AdminLogin,
    #[route("/admin/register")]
    AdminRegister,
    #[route("/add")]
    AddMember,
}
