use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    //Arrange
    let app = spawn_app().await;

    //Act
    let response = app.get_change_password().await;

    //Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    //Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    //Act
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    //Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    //Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    //Act - login
    app.post_login(&serde_json::json!({

        "username" : &app.test_user.username,
        "password" : &app.test_user.password
    }))
    .await;

    //Act - try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &another_new_password,
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");
    //Act - follow the redirect
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - \
        the field values must match.</i></p>"
    ))
}

#[tokio::test]
async fn current_password_must_be_valid() {
    //Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    //Act - login
    app.post_login(&serde_json::json!({

        "username" : &app.test_user.username,
        "password" : &app.test_user.password
    }))
    .await;

    //Act - try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &wrong_password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    //Assert
    assert_is_redirect_to(&response, "/admin/password");

    //Act - follow the redirect
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"))
}

#[tokio::test]
async fn new_password_must_be_longer_than_12_graphemes() {
    //Arrange
    let app = spawn_app().await;
    let new_password = "123456789012";

    //Act - login
    app.post_login(&serde_json::json!({
        "username" : &app.test_user.username,
        "password" : &app.test_user.password
    }))
    .await;

    //Act - try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    //Assert
    assert_is_redirect_to(&response, "/admin/password");

    //Act - follow the redirect
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("<p><i>Password length must be > 12  and < 128</i></p>"))
}

#[tokio::test]
async fn new_password_must_be_shorter_than_128_graphemes() {
    //Arrange
    let app = spawn_app().await;
    let new_password = "x".repeat(128);

    //Act - login
    app.post_login(&serde_json::json!({
        "username" : &app.test_user.username,
        "password" : &app.test_user.password
    }))
    .await;

    //Act - try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    //Assert
    assert_is_redirect_to(&response, "/admin/password");

    //Act - follow the redirect
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("<p><i>Password length must be > 12  and < 128</i></p>"))
}

#[tokio::test]
async fn changing_password_works() {
    //Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4();

    //Act - login
    let response = app
        .post_login(&serde_json::json!({
            "username" : &app.test_user.username,
            "password" : &app.test_user.password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    //Act - try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    //Act - follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed.</i></p>"));

    //Act - logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    //Act - follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    // Act - Log back in with the new password
    let response = app
        .post_login(&serde_json::json!({
            "username" : &app.test_user.username,
            "password" : &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/dashboard");
}
