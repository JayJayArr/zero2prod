use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run().await.unwrap().await.unwrap();
    Ok(())
}
