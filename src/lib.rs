use axum::{
    Form, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    serve::Serve,
};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Deserialize, Debug)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn subscribe_handler(Form(sign_up): Form<FormData>) -> impl IntoResponse {
    println!("email: {:?}, name: {:?}", sign_up.email, sign_up.name);
    StatusCode::OK
}

pub fn run(listener: TcpListener) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check_handler))
        .route("/subscriptions", post(subscribe_handler));
    // let listener = tokio::net::TcpListener::bind(listener).await?;

    Ok(axum::serve(listener, app))
}
