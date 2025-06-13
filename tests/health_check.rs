#[tokio::test]
async fn health_check_works() {
    spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:3000/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

async fn spawn_app() {
    let server = zero2prod::run().await.unwrap();

    let _handle = tokio::spawn(async move { server.await.unwrap() });

    // let _ = tokio::join!(handle);
}
