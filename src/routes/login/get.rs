use axum::response::{Html, IntoResponse};
use reqwest::StatusCode;

pub async fn login_form() -> impl IntoResponse {
    (StatusCode::OK, Html(include_str!("login.html")))
}
