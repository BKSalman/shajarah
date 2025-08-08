use crate::api::{
    members::{
        models::MemberInviteResponse,
        routes::{get_member_invites, members_count, InvitesParams},
    },
    users::models::UserResponseBrief,
};
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::{
        members::{
            models::{MemberResponseBrief, RequestedMemberResponseBrief},
            routes::{get_members_flat, get_requested_members_flat, FlatMembersParams},
            MembersError,
        },
        users::models::UserRole,
    },
    auth::{AuthError, AuthExtractor},
    InnerAppState,
};

mod filters {
    use base64::Engine;

    pub fn bytes_to_base64(bytes: &[u8]) -> ::askama::Result<String> {
        Ok(base64::prelude::BASE64_STANDARD.encode(bytes))
    }

    pub fn get_initials(name: &str, last_name: &str) -> ::askama::Result<String> {
        let first_char = name.chars().next().unwrap_or('؟');
        let last_char = if last_name != name && !last_name.is_empty() {
            last_name.chars().next().unwrap_or(' ')
        } else {
            ' '
        };

        if last_char != ' ' {
            Ok(format!("{}{}", first_char, last_char))
        } else {
            Ok(first_char.to_string())
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PagesError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Members(#[from] MembersError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("")]
    NotFound,
}

impl IntoResponse for PagesError {
    fn into_response(self) -> axum::response::Response {
        log::error!("{self:#?}");

        match self {
            PagesError::Auth(e) => e.into_response(),
            PagesError::Members(e) => e.into_response(),
            PagesError::NotFound => NotFoundTemplate.into_response(),
            PagesError::Sqlx(_) => SomethingWentWrongTemplate.into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplateInner;

pub struct NotFoundTemplate;

impl axum::response::IntoResponse for NotFoundTemplate {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::NOT_FOUND, NotFoundTemplateInner).into_response()
    }
}

#[derive(Template)]
#[template(path = "500.html")]
struct SomethingWentWrongTemplateInner;

pub struct SomethingWentWrongTemplate;

impl axum::response::IntoResponse for SomethingWentWrongTemplate {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            SomethingWentWrongTemplateInner,
        )
            .into_response()
    }
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    name: String,
    members: Vec<MemberResponseBrief>,
    members_count: usize,
    add_requests: Vec<RequestedMemberResponseBrief>,
    member_invites: Vec<MemberInviteResponse>,
    members_query: Option<String>,
    requests_query: Option<String>,
    members_page: usize,
    members_per_page: usize,
    requests_page: usize,
    requests_per_page: usize,
}

impl AdminTemplate {
    pub fn members_json(&self) -> String {
        serde_json::to_string(&self.members).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn add_requests_json(&self) -> String {
        serde_json::to_string(&self.add_requests).unwrap_or_else(|_| "[]".to_string())
    }
}

serde_with::with_prefix!(prefix_members "members_");
serde_with::with_prefix!(prefix_requests "requests_");
serde_with::with_prefix!(prefix_invites "invites_");

#[derive(Deserialize)]
pub struct AdminParams {
    #[serde(flatten, with = "prefix_members")]
    members_params: FlatMembersParams,
    #[serde(flatten, with = "prefix_requests")]
    requests_params: FlatMembersParams,
    #[serde(flatten, with = "prefix_invites")]
    invite_params: InvitesParams,
}

pub async fn admin_page(
    auth: Result<AuthExtractor<{ UserRole::Admin as u8 }>, AuthError>,
    state: State<Arc<InnerAppState>>,
    Query(params): Query<AdminParams>,
) -> Result<AdminTemplate, PagesError> {
    match auth {
        Ok(auth) => {
            let members_query = params.members_params.query.clone();
            let members_page = params.members_params.page.unwrap_or(0);
            let members_per_page = params.members_params.per_page.unwrap_or(12);
            let Json(members) =
                match get_members_flat(state.clone(), Query(params.members_params)).await {
                    Ok(members) => members,
                    Err(MembersError::NoMembers) => Vec::new().into(),
                    Err(e) => return Err(e.into()),
                };
            let requests_query = params.requests_params.query.clone();
            let requests_page = params.requests_params.page.unwrap_or(0);
            let requests_per_page = params.requests_params.per_page.unwrap_or(12);
            let Json(add_requests) =
                get_requested_members_flat(state.clone(), Query(params.requests_params)).await?;

            let name = auth.current_user.first_name.clone();

            let Json(member_invites) =
                get_member_invites(auth, state.clone(), Query(params.invite_params)).await?;

            let members_count = members_count(state).await? as usize;

            Ok(AdminTemplate {
                name,
                members,
                add_requests,
                member_invites,
                members_query,
                requests_query,
                members_page,
                members_per_page,
                requests_page,
                requests_per_page,
                members_count,
            })
        }
        Err(e) => match e {
            AuthError::InvalidSession | AuthError::SessionError(_) => Err(PagesError::Auth(e)),
            e => Err(e.into()),
        },
    }
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct AdminLoginTemplate;

pub async fn admin_login_page(
    auth: Option<Result<AuthExtractor<{ UserRole::Admin as u8 }>, AuthError>>,
) -> impl IntoResponse {
    // if already logged in, redirect to admin page
    if auth.is_some_and(|a| a.is_ok()) {
        return Redirect::to("/admin").into_response();
    }

    AdminLoginTemplate.into_response()
}

#[derive(Template)]
#[template(path = "members-login.html")]
pub struct MembersLoginTemplate;

pub async fn members_login_page() -> impl IntoResponse {
    MembersLoginTemplate.into_response()
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate;

pub async fn admin_register_page(
    state: State<Arc<InnerAppState>>,
) -> Result<impl IntoResponse, PagesError> {
    if sqlx::query!(
        r#"
            SELECT id, role as "role: UserRole" FROM users
            WHERE role = $1
        "#,
        UserRole::Admin as _,
    )
    .fetch_optional(&state.db_pool)
    .await?
    .is_some()
    {
        return Err(PagesError::NotFound);
    }

    Ok(RegisterTemplate)
}

#[derive(Template)]
#[template(path = "add-request.html")]
pub struct AddRequestTemplate {
    members: Vec<MemberResponseBrief>,
}

pub async fn add_request_page(
    state: State<Arc<InnerAppState>>,
    params: Query<FlatMembersParams>,
) -> Result<AddRequestTemplate, PagesError> {
    let Json(members) = match get_members_flat(state, params).await {
        Ok(members) => members,
        Err(MembersError::NoMembers) => Vec::new().into(),
        Err(e) => return Err(e.into()),
    };

    Ok(AddRequestTemplate { members })
}

#[derive(Template)]
#[template(path = "invite.html")]
pub struct InviteTemplate {
    invite_id: Uuid,
}

pub async fn invite_reply_page(
    state: State<Arc<InnerAppState>>,
    Path(invite_id): Path<Uuid>,
) -> Result<InviteTemplate, PagesError> {
    let query = sqlx::query!(
        r#"SELECT id FROM member_invites WHERE id = $1 AND status = 'pending'"#,
        invite_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if query.is_some() {
        Ok(InviteTemplate { invite_id })
    } else {
        Err(PagesError::NotFound)
    }
}

#[derive(Template)]
#[template(path = "user-page.html")]
pub struct UserPageTemplate {
    current_user: UserResponseBrief,
}

pub async fn user_page(
    auth: AuthExtractor<{ UserRole::User as u8 }>,
    _state: State<Arc<InnerAppState>>,
) -> Result<UserPageTemplate, PagesError> {
    Ok(UserPageTemplate {
        current_user: auth.current_user,
    })
}
