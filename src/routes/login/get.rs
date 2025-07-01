use axum::response::{Html, IntoResponse};
use axum_messages::{Level, Messages};
use std::fmt::Write;

#[axum::debug_handler]
pub async fn login_form(messages: Messages) -> impl IntoResponse {
    let mut error_html = String::new();
    for m in messages.into_iter().filter(|m| m.level == Level::Error) {
        writeln!(error_html, "<p><i>{m}</i></p>").unwrap();
    }

    Html(format!(
        r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Login</title>
            </head>
            <body>
                {error_html}
                <form action="/login" method="post">
                    <label>Username
                        <input
                            type="text"
                            placeholder="Enter Username"
                            name="username"
                        >
                    </label>
                    <label>Password
                        <input
                            type="password"
                            placeholder="Enter Password"
                            name="password"
                        >
                    </label>
                    <button type="submit">Login</button>
                </form>
            </body>
            </html>"#
    ))
}
