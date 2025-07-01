use axum::response::{Html, IntoResponse};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;

#[axum::debug_handler]
pub async fn login_form(jar: CookieJar) -> impl IntoResponse {
    dbg!(&jar);
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let body = Html(format!(
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
    ));

    (jar.add(Cookie::new("_flash", "")), body)
}
