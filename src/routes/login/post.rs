use axum::{
    Form,
    extract::State,
    http::status::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_messages::Messages;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    startup::AppState,
};

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[axum::debug_handler]
pub async fn login(
    state: State<AppState>,
    messages: Messages,
    formdata: Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: formdata.username.clone(),
        password: formdata.password.clone(),
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &state.pg_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));

            Ok(Redirect::to("/login"))
        }

        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            messages.error(e.to_string());
            Ok(Redirect::to("/login"))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            LoginError::AuthError(rejection) => (StatusCode::UNAUTHORIZED, rejection.to_string()),
            LoginError::UnexpectedError(err) => {
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
