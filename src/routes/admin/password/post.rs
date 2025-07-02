use axum::Form;
use reqwest::StatusCode;
use secrecy::SecretString;
use serde::Deserialize;

use crate::routes::{PasswordError, session_state::TypedSession};

#[derive(Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

#[axum::debug_handler]
pub async fn change_password(
    session: TypedSession,
    Form(form): Form<FormData>,
) -> Result<StatusCode, PasswordError> {
    if session
        .get_user_id()
        .await
        .map_err(|e| PasswordError::UnexpectedError(e.into()))?
        .is_none()
    {
        return Err(PasswordError::Unauthenticated("".into()));
    } else {
        todo!()
    }
}
