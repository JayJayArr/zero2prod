mod get;
mod post;

use axum::response::{IntoResponse, Redirect, Response};
pub use get::change_password_form;
pub use post::change_password;
use reqwest::StatusCode;

impl IntoResponse for PasswordError {
    fn into_response(self) -> Response {
        match self {
            PasswordError::ValidationError(rejection) => {
                (StatusCode::BAD_REQUEST, rejection).into_response()
            }
            PasswordError::UnexpectedError(err) => {
                tracing::error!("{:?}", err);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
                    .into_response()
            }
            PasswordError::Unauthenticated(_) => Redirect::to("/login").into_response(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("{0}")]
    Unauthenticated(String),
}
