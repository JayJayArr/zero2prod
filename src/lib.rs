use axum::{Router, http::StatusCode, response::IntoResponse, routing::get, serve::Serve};
use tokio::net::TcpListener;

async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn run(
    listener: TcpListener,
) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let app = Router::new().route("/health_check", get(health_check_handler));
    // let listener = tokio::net::TcpListener::bind(listener).await?;

    Ok(axum::serve(listener, app))
}
