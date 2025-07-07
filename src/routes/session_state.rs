use anyhow::Result;
use axum::{extract::FromRequestParts, http::request::Parts, response::Redirect};
use axum_login::tower_sessions::Session;
use uuid::Uuid;

#[derive(Debug)]
pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub async fn cycle_id(&self) {
        let _ = self.0.cycle_id().await;
    }

    pub async fn insert_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<(), axum_login::tower_sessions::session::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id).await
    }

    pub async fn get_user_id(
        &self,
    ) -> Result<Option<Uuid>, axum_login::tower_sessions::session::Error> {
        self.0.get(Self::USER_ID_KEY).await
    }
    pub async fn log_out(
        &self,
    ) -> Result<Option<Uuid>, axum_login::tower_sessions::session::Error> {
        self.0.remove(Self::USER_ID_KEY).await
    }
}

impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;
        let user_id: Uuid = match session.get(Self::USER_ID_KEY).await.unwrap() {
            Some(id) => id,
            None => return Err(Redirect::to("/login")),
        };
        session.insert(Self::USER_ID_KEY, user_id).await.unwrap();

        Ok(Self(session))
    }
}
