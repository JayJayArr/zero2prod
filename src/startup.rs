use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    serve::Serve,
};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::routes::{health_check_handler, subscribe_handler};

#[derive(Clone, Debug)]
pub struct AppState {
    pub pg_pool: Arc<PgPool>,
}

pub fn run(
    listener: TcpListener,
    connection: PgPool,
) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let state = AppState {
        pg_pool: Arc::new(connection),
    };
    let app = Router::new()
        .route("/health_check", get(health_check_handler))
        .route("/subscriptions", post(subscribe_handler))
        .with_state(state);

    Ok(axum::serve(listener, app))
}
