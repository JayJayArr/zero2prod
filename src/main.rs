use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use zero2prod::{
    configuration::get_configuration,
    issue_delivery_worker::run_worker_until_stopped,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //Setup Tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    //Load Config
    let configuration = get_configuration().expect("failed to read config");
    //create postgres connection pool from config

    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration.clone()));
    tracing::info!(
        "Starting application with following config {:?}",
        configuration
    );

    tokio::select! {o = application_task => {report_exit("API", o);}, o = worker_task => {report_exit("Background workder", o);}}
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} failed", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} task failed to complete", task_name)
        }
    }
}
