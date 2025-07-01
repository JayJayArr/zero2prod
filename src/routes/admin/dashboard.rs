use anyhow::Context;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{routes::session_state::TypedSession, startup::AppState};

#[axum::debug_handler]
pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: TypedSession,
) -> impl IntoResponse {
    let username = if let Some(user_id) = session
        .get_user_id()
        .await
        .expect("failed to get user_id from session.")
    {
        get_username(user_id, &state.pg_pool)
            .await
            .map_err(|_| "".to_string())
            .expect("Failed getting username")
    } else {
        "".to_string()
    };

    let body = Html(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Admin dashboard</title>
        </head>
        <body>
            <p>Welcome {username}!</p>
        </body>
        </html>"#
    ));

    body
}

#[tracing::instrument(name = "Get username", skip(pool))]
async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
    "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username")?;
    Ok(row.username)
}
