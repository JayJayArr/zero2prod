use anyhow::Context;
use argon2::PasswordHasher;
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version, password_hash::SaltString,
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

use crate::telemetry::spawn_blocking_with_tracing;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, SecretString)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials")?
    .map(|row| (row.user_id, SecretString::new(row.password_hash.into())));

    Ok(row)
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = SecretString::new(
        "$argon2id$v=19$m=15000,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".into()
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Validate credentials",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Validate credentials", skip(password, pool))]
pub async fn change_password(
    user_id: uuid::Uuid,
    password: SecretString,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")?;

    sqlx::query!(
        r#"UPDATE users SET password_hash = $1 WHERE user_id = $2"#,
        password_hash.expose_secret(),
        user_id
    )
    .execute(pool)
    .await
    .context("Failed to change user's passwor in the database.")?;
    Ok(())
}

fn compute_password_hash(password: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(SecretString::new(password_hash.into()))
}
