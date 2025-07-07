use axum::response::IntoResponse;
use axum_messages::Messages;

use crate::routes::{PublishError, session_state::TypedSession};
use std::fmt::Write;

pub async fn publish_newsletters_form(
    session: TypedSession,
    messages: Messages,
) -> Result<impl IntoResponse, PublishError> {
    if session
        .get_user_id()
        .await
        .expect("failed to get user_id from session.")
        .is_some()
    {
        let mut msg_html = String::new();
        for m in messages.into_iter() {
            writeln!(msg_html, "<p><i>{m}</i></p>").unwrap();
        }
        let body = format!(
            r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Publish Newsletter Issue</title>
                </head>
                <body>
                    {msg_html}
                    <form action="/admin/newsletters" method="post">
                        <label>Title:<br>
                            <input
                                type="text"
                                placeholder="Enter the issue title"
                                name="title"
                            >
                        </label>
                        <br>
                        <label>Plain text content:<br>
                            <textarea
                                placeholder="Enter the content in plain text"
                                name="text_content"
                                rows="20"
                                cols="50"
                            ></textarea>
                        </label>
                        <br>
                        <label>HTML content:<br>
                            <textarea
                                placeholder="Enter the content in HTML format"
                                name="html_content"
                                rows="20"
                                cols="50"
                            ></textarea>
                        </label>
                        <br>
                        <button type="submit">Publish</button>
                    </form>
                    <p><a href="/admin/dashboard">&lt;- Back</a></p>
                </body>
                </html>"#
        );
        Ok(body)
    } else {
        Err(PublishError::Unauthenticated("Please log in.".into()))
    }
}
