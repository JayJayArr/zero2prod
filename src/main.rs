use tokio::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("could not bind random port");
    run(listener).await.unwrap().await.unwrap();
    Ok(())
}
