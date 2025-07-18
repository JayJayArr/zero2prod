use anyhow::Context;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{routes::session_state::TypedSession, startup::AppState};

pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: TypedSession,
) -> Result<impl IntoResponse, Redirect> {
    let username = if let Some(user_id) = session
        .get_user_id()
        .await
        .expect("failed to get user_id from session.")
    {
        get_username(user_id, &state.pg_pool)
            .await
            .map_err(|_| Redirect::to("/login"))
    } else {
        return Err(Redirect::to("/login"));
    };

    let usernamestring = match username {
        Ok(username) => username,
        Err(_) => {
            return Err(Redirect::to("/login"));
        }
    };

    let body = Html(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Admin dashboard</title>
        </head>
        <body>
            <p>Welcome {usernamestring}!</p>
            <p>Available actions:</p>
            <ol>
                <li><a href="/admin/password">Change password</a></li>
                <li><a href="/admin/newsletters">Send a newsletter</a></li>
                <li>
                    <form name="logoutForm" action="/admin/logout" method="post">
                        <input type="submit" value"Logout">
                    </form>
                </li>
            </ol>
        </body>
        </html>"#
    ));

    Ok(body)
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
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
