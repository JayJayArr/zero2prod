use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //Setup Tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    //Load Config
    let configuration = get_configuration().expect("failed to read config");
    //create postgres connection pool from config
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .expect("could not bind port");
    //Start the application
    run(listener, connection_pool)
        .expect("failed running app")
        .await
        .unwrap();
    Ok(())
}
