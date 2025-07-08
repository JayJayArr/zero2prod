use anyhow::Context;
use axum::{
    Form,
    extract::State,
    http::status::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    domain::SubscriberEmail,
    idempotency::{IdempotencyKey, get_saved_response, save_response},
    routes::session_state::TypedSession,
    startup::AppState,
};

#[derive(Deserialize)]
pub struct FormData {
    title: String,
    html: String,
    text: String,
    idempotency_key: String,
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    Unauthenticated(String),
    #[error("{0}")]
    ValidationError(String),
}

#[tracing::instrument(
    name = "Publishing new newsletter",
    skip(session, messages, state, form)
)]
pub async fn publish_newsletters_handler(
    session: TypedSession,
    messages: Messages,
    state: State<AppState>,
    form: Form<FormData>,
) -> Result<impl IntoResponse, PublishError> {
    if let Some(user_id) = session
        .get_user_id()
        .await
        .expect("failed to get user_id from session.")
    {
        let idempotency_key: IdempotencyKey = form
            .idempotency_key
            .clone()
            .try_into()
            .map_err(|e: anyhow::Error| PublishError::ValidationError(e.to_string()))?;
        if let Some(saved_response) = get_saved_response(&state.pg_pool, &idempotency_key, user_id)
            .await
            .map_err(|e| PublishError::ValidationError(e.to_string()))?
        {
            messages.info("The newsletter issue has been published!");
            return Ok(saved_response);
        }
        let subscribers = get_confirmed_subscribers(&state.pg_pool).await?;

        for subscriber in subscribers {
            match subscriber {
                Ok(subscriber) => {
                    state
                        .email_client
                        .send_email(&subscriber.email, &form.title, &form.html, &form.text)
                        .await
                        .with_context(|| {
                            format!("Failed to send newsletter issue to {}", subscriber.email)
                        })?;
                }
                Err(error) => {
                    tracing::warn!(
                        "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid: {}",
                        error
                    );
                }
            }
        }
        messages.info("The newsletter issue has been published!");
        let response = Redirect::to("/admin/newsletters").into_response();
        let response = save_response(&state.pg_pool, &idempotency_key, user_id, response)
            .await
            .map_err(|e| PublishError::UnexpectedError(e))?;
        Ok(response)
    } else {
        return Err(PublishError::Unauthenticated("Please log in.".into()));
    }
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!("SELECT email FROM subscriptions WHERE status = 'confirmed'")
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|r| match SubscriberEmail::parse(r.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(error) => Err(anyhow::anyhow!(error)),
            })
            .collect();
    Ok(confirmed_subscribers)
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            PublishError::UnexpectedError(err) => {
                tracing::error!("{:?}", err);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            PublishError::Unauthenticated(e) => (StatusCode::UNAUTHORIZED, e),
            PublishError::ValidationError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        (status, message).into_response()
    }
}

impl From<sqlx::Error> for PublishError {
    fn from(value: sqlx::Error) -> Self {
        Self::UnexpectedError(value.into())
    }
}
