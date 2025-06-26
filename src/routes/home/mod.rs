use axum::response::{Html, IntoResponse};
use reqwest::StatusCode;

pub async fn home() -> impl IntoResponse {
    (StatusCode::OK, Html(include_str!("home.html")))
}
