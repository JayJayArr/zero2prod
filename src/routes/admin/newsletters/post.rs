use crate::{
    idempotency::{IdempotencyKey, NextAction, save_response, try_processing},
    routes::session_state::TypedSession,
    startup::AppState,
};
use anyhow::Context;
use axum::{
    Form,
    extract::State,
    http::status::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use serde::Deserialize;
use sqlx::Executor;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    title: String,
    html: String,
    text: String,
    idempotency_key: String,
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

        let mut transaction = match try_processing(&state.pg_pool, &idempotency_key, user_id)
            .await
            .map_err(PublishError::UnexpectedError)?
        {
            NextAction::StartProcessing(t) => t,
            NextAction::ReturnSavedResponse(saved_response) => {
                messages.info("The newsletter issue has been published!");
                return Ok(saved_response);
            }
        };
        let issue_id =
            insert_newsletter_issue(&mut transaction, &form.title, &form.text, &form.html)
                .await
                .context("Failed to store newsletter issue details")
                .map_err(PublishError::UnexpectedError)?;
        enqueue_deliver_tasks(&mut transaction, issue_id)
            .await
            .context("Failed to enqueue delivery tasks")
            .map_err(PublishError::UnexpectedError)?;

        //Old
        messages.info("The newsletter issue has been published!");
        let response = Redirect::to("/admin/newsletters").into_response();
        let response = save_response(transaction, &idempotency_key, user_id, response)
            .await
            .map_err(PublishError::UnexpectedError)?;
        Ok(response)
    } else {
        return Err(PublishError::Unauthenticated("Please log in.".into()));
    }
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

#[tracing::instrument(name = "insert newsletter issue", skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'static, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
    INSERT INTO newsletter_issues (
        newsletter_issue_id,
        title, 
        text_content,
        html_content,
        published_at
    )
    VALUES ($1, $2, $3, $4, now())
    "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}

async fn enqueue_deliver_tasks(
    transaction: &mut Transaction<'static, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
            INSERT INTO issue_delivery_queue (
                newsletter_issue_id,
                subscriber_email
            )
            SELECT $1, email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    );
    transaction.execute(query).await?;
    Ok(())
}

impl From<sqlx::Error> for PublishError {
    fn from(value: sqlx::Error) -> Self {
        Self::UnexpectedError(value.into())
    }
}
