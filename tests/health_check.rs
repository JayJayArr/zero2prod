use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    //Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    //Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("", "missing both name and email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("name=le%20guin", "missing the email"),
    ];
    //Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");
        println!("{:?}", response);
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("could not bind random port");

    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).unwrap();

    let _handle = tokio::spawn(async move { server.await.unwrap() });

    // let _ = tokio::join!(handle);
    format!("http://127.0.0.1:{}", port)
}
