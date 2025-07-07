use axum::response::{IntoResponse, Redirect};
use axum_messages::Messages;
use reqwest::StatusCode;

use crate::routes::session_state::TypedSession;

#[derive(thiserror::Error, Debug)]
pub enum LogoutError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for LogoutError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            LogoutError::AuthError(rejection) => (StatusCode::UNAUTHORIZED, rejection.to_string()),
            LogoutError::UnexpectedError(err) => {
                tracing::error!("{:?}", err);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
        };

        (status, message).into_response()
    }
}

pub async fn log_out(
    messages: Messages,
    session: TypedSession,
) -> Result<impl IntoResponse, LogoutError> {
    if session
        .get_user_id()
        .await
        .map_err(|e| LogoutError::AuthError(e.into()))?
        .is_none()
    {
        Ok(Redirect::to("/login"))
    } else {
        session
            .log_out()
            .await
            .map_err(|e| LogoutError::UnexpectedError(e.into()))?;
        messages.info("You have successfully logged out.");
        Ok(Redirect::to("/login"))
    }
}
