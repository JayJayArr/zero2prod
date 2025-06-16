use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use tracing::Instrument;
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
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
    "Adding a new subscriber.",
    %request_id,
    subscriber_email = %form.email,
    subscriber_name= %form.name
    );

    let _request_span_guard = request_span.enter();
    tracing::info!(
        "request:if {} - Adding '{}' '{}' as a subscriber",
        request_id,
        form.email,
        form.name
    );
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query: {}", request_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
