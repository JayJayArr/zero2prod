use axum::{
    Form,
    extract::State,
    http::{HeaderMap, status::StatusCode},
    response::IntoResponse,
};
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    authentication::{Credentials, validate_credentials},
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

            let mut headers = HeaderMap::new();
            headers.insert(
                "Location",
                "/".parse().expect("Failed to set location header"),
            );
            Ok((StatusCode::SEE_OTHER, headers))
        }

        Err(_) => {
            todo!()
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
