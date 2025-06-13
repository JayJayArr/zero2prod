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

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("could not bind random port");

    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).await.unwrap();

    let _handle = tokio::spawn(async move { server.await.unwrap() });

    // let _ = tokio::join!(handle);
    format!("http://127.0.0.1:{}", port)
}
