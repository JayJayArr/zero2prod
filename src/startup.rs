use axum::{
    Router,
    routing::{get, post},
    serve::Serve,
};
use tokio::net::TcpListener;

use crate::routes::{health_check_handler, subscribe_handler};

pub fn run(listener: TcpListener) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check_handler))
        .route("/subscriptions", post(subscribe_handler));
    // let listener = tokio::net::TcpListener::bind(listener).await?;

    Ok(axum::serve(listener, app))
}
