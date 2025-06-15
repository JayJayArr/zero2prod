use axum::{Form, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}
pub async fn subscribe_handler(Form(sign_up): Form<FormData>) -> impl IntoResponse {
    println!("email: {:?}, name: {:?}", sign_up.email, sign_up.name);
    StatusCode::OK
}
