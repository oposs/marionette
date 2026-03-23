use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use marionette::db_session as session;
use marionette::ws::AppState;

use crate::entities::user;

/// Login request payload.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Handle user login via HTTP POST.
///
/// Validates credentials, creates a session row, and returns an HTTP-only cookie.
///
/// # Errors
///
/// Returns `UNAUTHORIZED` for invalid credentials, `INTERNAL_SERVER_ERROR` for DB errors.
#[allow(clippy::missing_errors_doc)]
pub async fn handle_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    // 1. Look up user by email
    let found_user = user::Entity::find()
        .filter(user::Column::UserEmail.eq(&req.username))
        .one(&*state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Verify password (spawn_blocking for CPU-bound bcrypt)
    let hash = found_user.user_password.clone();
    let password = req.password.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. Create session token
    let token = uuid::Uuid::new_v4().to_string();
    let expires = (Utc::now() + chrono::Duration::hours(24))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // 4. Insert session row
    let session_row = session::ActiveModel {
        session_token: Set(token.clone()),
        session_user: Set(Some(found_user.user_id)),
        session_roles: Set(serde_json::json!(vec![&found_user.user_role]).to_string()),
        session_created: Set(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        session_expires: Set(expires),
        ..Default::default()
    };
    session_row
        .insert(&*state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 5. Update user_last_login
    let mut active_user: user::ActiveModel = found_user.into();
    active_user.user_last_login = Set(Some(
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    ));
    active_user
        .update(&*state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6. Build cookie
    let cookie = Cookie::build(("marionette_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(24))
        .build();

    Ok((jar.add(cookie), Json(serde_json::json!({"ok": true}))))
}

/// Handle user logout via HTTP POST.
///
/// Clears the session cookie and deletes the session row from the database.
pub async fn handle_logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> (CookieJar, StatusCode) {
    if let Some(cookie) = jar.get("marionette_session") {
        let token = cookie.value().to_owned();
        // Delete session row from DB (best effort)
        let _ = session::Entity::delete_many()
            .filter(session::Column::SessionToken.eq(&token))
            .exec(&*state.db)
            .await;
    }

    let removal = Cookie::build("marionette_session")
        .path("/")
        .build();

    (jar.remove(removal), StatusCode::OK)
}
