use anyhow::Context;
use axum::{
    Json,
    extract::State,
    http::status::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, routes::session_state::TypedSession, startup::AppState};

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
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
}

#[tracing::instrument(name = "Publishing new newsletter", skip(session, state, body))]
pub async fn pubslish_newsletters_handler(
    session: TypedSession,
    state: State<AppState>,
    body: Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    if session
        .get_user_id()
        .await
        .expect("failed to get user_id from session.")
        .is_some()
    {
        let subscribers = get_confirmed_subscribers(&state.pg_pool).await?;

        for subscriber in subscribers {
            match subscriber {
                Ok(subscriber) => {
                    state
                        .email_client
                        .send_email(
                            &subscriber.email,
                            &body.title,
                            &body.content.html,
                            &body.content.text,
                        )
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
        return Ok(StatusCode::OK);
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
        };

        (status, message).into_response()
    }
}

impl From<sqlx::Error> for PublishError {
    fn from(value: sqlx::Error) -> Self {
        Self::UnexpectedError(value.into())
    }
}
