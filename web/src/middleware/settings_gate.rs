use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{modules::settings::server::add_page_enabled, server::InnerAppState};

pub async fn gate_add_page(
    State(state): State<Arc<InnerAppState>>,
    request: Request,
    next: Next,
) -> Response {
    let gated = matches!(
        (request.method(), request.uri().path()),
        (&Method::GET, "/add") | (&Method::POST, "/api/v1/members/request")
    );

    if gated {
        match add_page_enabled(&state.db_pool).await {
            Ok(true) => {}
            Ok(false) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                tracing::error!("failed to read add_page_enabled setting: {e}");
                return StatusCode::NOT_FOUND.into_response();
            }
        }
    }

    next.run(request).await
}
