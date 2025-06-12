use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn run() -> Result<(), std::io::Error> {
    let app = Router::new().route("/health_check", get(health_check_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await
}
