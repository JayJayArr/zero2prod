use crate::helpers::{ConfirmationLinks, TestApp, assert_is_redirect_to, spawn_app};
use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use wiremock::{
    Mock, MockBuilder, ResponseTemplate,
    matchers::{any, method, path},
};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    //Act - login
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4()
    });

    let response = app.post_newsletters(&newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    //Act - login
    app.test_user.login(&app).await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4()
    });

    let response = app.post_newsletters(&newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    let app = spawn_app().await;

    //Act - login
    app.test_user.login(&app).await;

    let test_cases = vec![
        (
            (serde_json::json!({
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
            "idempotency_key": uuid::Uuid::new_v4()
            }
            )),
            "missing title",
        ),
        (
            serde_json::json!({
            "title": "Newsletter title",

                }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(&invalid_body).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "The API did not fail with 400 Bad Request when the payload was {error_message}."
        );
    }
}

#[tokio::test]
async fn you_have_to_be_logged_in_to_post_a_newsletter() {
    let app = spawn_app().await;
    //Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4()
    });
    //Assert
    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_have_to_be_logged_in_to_view_the_post_site() {
    let app = spawn_app().await;
    //Act
    let response = app.get_newsletter().await;
    //Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    //Act submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    //Act - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));

    //Act - submit newsletter form **again**
    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    //Act - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));

    app.dispatch_all_pending_emails().await;
    //Assert on drop that the issues was sent only **once**
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    //Act submit newsletter form concurrently
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response1 = app.post_newsletters(&newsletter_request_body);
    let response2 = app.post_newsletters(&newsletter_request_body);

    let (response1, response2) = tokio::join!(response1, response2);
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await;

    //Assert on drop that only one mail was sent
}

// #[tokio::test]
// async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
//     //Arrange
//     let app = spawn_app().await;
//     //Two subscribers instead of one
//     create_confirmed_subscriber(&app).await;
//     create_confirmed_subscriber(&app).await;
//     app.test_user.login(&app).await;
//     let newsletter_request_body = serde_json::json!({
//         "title": "Newsletter title",
//         "text": "Newsletter body as plain text",
//         "html": "<p>Newsletter body as HTML</p>",
//         "idempotency_key": uuid::Uuid::new_v4().to_string()
//     });
//     app.test_user.login(&app).await;
//
//     //Act - Submit newsletter form
//     //Email delivery fails for the second subscriber
//     when_sending_an_email()
//         .respond_with(ResponseTemplate::new(200))
//         .up_to_n_times(1)
//         .expect(1)
//         .mount(&app.email_server)
//         .await;
//     when_sending_an_email()
//         .respond_with(ResponseTemplate::new(500))
//         .up_to_n_times(1)
//         .expect(1)
//         .mount(&app.email_server)
//         .await;
//     let response = app.post_newsletters(&newsletter_request_body).await;
//     assert_eq!(response.status().as_u16(), 500);
//
//     //Act - Retry submitting the form
//     //Email delivery will succeed for both subscribers now
//     when_sending_an_email()
//         .respond_with(ResponseTemplate::new(200))
//         .expect(1)
//         .named("Delivery retried")
//         .mount(&app.email_server)
//         .await;
//     let response = app.post_newsletters(&newsletter_request_body).await;
//     assert_eq!(response.status().as_u16(), 303);
// }

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
