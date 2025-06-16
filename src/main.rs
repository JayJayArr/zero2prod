use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read config");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .expect("could not bind port");
    run(listener, connection_pool)
        .expect("failed running app")
        .await
        .unwrap();
    Ok(())
}
