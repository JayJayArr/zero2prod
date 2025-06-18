use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;
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
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("could not bind port");
    info!(
        "Starting app on {:?}:{:?}",
        configuration.application.host, configuration.application.port
    );
    //Start the application
    run(listener, connection_pool)
        .expect("failed running app")
        .await
        .unwrap();
    Ok(())
}
