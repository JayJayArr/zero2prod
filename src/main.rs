use tokio::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("could not bind port");
    run(listener).expect("failed running app").await.unwrap();
    Ok(())
}
