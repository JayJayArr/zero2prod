use axum::response::{Html, IntoResponse};

use crate::routes::{PasswordError, session_state::TypedSession};

pub async fn change_password_form(
    session: TypedSession,
) -> Result<impl IntoResponse, PasswordError> {
    let userid = session
        .get_user_id()
        .await
        .map_err(|e| PasswordError::UnexpectedError(e.into()))?;
    dbg!(&userid);
    if userid.is_some() {
        dbg!("session not found");
        Ok(Html({
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Change Password</title>
            </head>
            <body>
                <form action="/admin/password" method="post">
                    <label>Current password
                        <input
                        type="password"
                        placeholder="Enter current password"
                        name="current_password"
                        >
                    </label>
                    <br>
                    <label>New password
                        <input
                        type="password"
                        placeholder="Enter new password"
                        name="new_password"
                        >
                    </label>
                    <br>
                    <label>Confirm new password
                        <input
                        type="password"
                        placeholder="Type the new password again"
                        name="new_password_check"
                        >
                    </label>
                    <br>
                    <button type="submit">Change password</button>
                </form>
                <p><a href="/admin/dashboard">&lt;- Back</a></p>
            </body>
            </html>"#
        }))
    } else {
        Err(PasswordError::Unauthenticated("please log in".to_string()))
    }
}
