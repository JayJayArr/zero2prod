use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe_handler(
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(state.pg_pool.as_ref())
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
