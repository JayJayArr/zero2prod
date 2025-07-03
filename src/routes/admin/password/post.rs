use axum::{Form, extract::State, response::Redirect};
use axum_messages::Messages;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    routes::{PasswordError, get_username, session_state::TypedSession},
    startup::AppState,
};

#[derive(Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

#[axum::debug_handler]
pub async fn change_password(
    messages: Messages,
    session: TypedSession,
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> Result<Redirect, PasswordError> {
    let user_id = session
        .get_user_id()
        .await
        .map_err(|e| PasswordError::UnexpectedError(e.into()))?;
    if let Some(userid) = user_id {
        let username = get_username(userid, &state.pg_pool).await?;
        let credentials = Credentials {
            username,
            password: form.current_password,
        };

        if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
            messages
                .error("You entered two different new passwords - the field values must match.");
            return Err(PasswordError::ValidationError(
                "passwords don't match".to_string(),
            ));
        }

        if form.new_password.expose_secret().len() <= 12
            || form.new_password.expose_secret().len() >= 128
        {
            messages.error("Password lenght must be > 12  and < 128");
            return Err(PasswordError::ValidationError(
                "password length invalid".to_string(),
            ));
        }
        if let Err(e) = validate_credentials(credentials, &state.pg_pool).await {
            return match e {
                AuthError::InvalidCredentials(_) => {
                    messages.error("The current password is incorrect.");

                    Err(PasswordError::ValidationError(
                        "credentials invalid".to_string(),
                    ))
                }
                AuthError::UnexpectedError(e) => Err(PasswordError::UnexpectedError(e)),
            };
        }
        messages.error("Your password has been changed.");
        Ok(Redirect::to("/admin/password"))
    } else {
        Err(PasswordError::Unauthenticated("".into()))
    }
}
