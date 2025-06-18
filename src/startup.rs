use axum::{
    Router,
    extract::{MatchedPath, Request},
    routing::{get, post},
    serve::Serve,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

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
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = uuid::Uuid::new_v4();
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                    request_id = tracing::field::display(request_id),
                )
            }),
        )
        .with_state(state);

    info!("Starting server");
    Ok(axum::serve(listener, app))
}
