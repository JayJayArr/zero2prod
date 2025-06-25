use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{health_check_handler, subscribe_handler, subscriptions_confirm_handler},
};
use axum::{
    Router,
    extract::{MatchedPath, Request},
    routing::{get, post},
    serve::Serve,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

#[derive(Clone, Debug)]
pub struct AppState {
    pub pg_pool: Arc<PgPool>,
    pub email_client: Arc<EmailClient>,
    pub base_url: Arc<ApplicationBaseUrl>,
}

pub struct Application {
    port: u16,
    server: Serve<TcpListener, Router, Router>,
}
#[derive(Clone, Debug)]
pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let state = AppState {
        pg_pool: Arc::new(connection),
        email_client: Arc::new(email_client),
        base_url: Arc::new(ApplicationBaseUrl(base_url)),
    };
    let app = Router::new()
        .route("/health_check", get(health_check_handler))
        .route("/subscriptions", post(subscribe_handler))
        .route("/subscriptions/confirm", get(subscriptions_confirm_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = uuid::Uuid::new_v4();
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        request_id = tracing::field::display(request_id),
                    )
                })
                .on_failure(()),
        )
        .with_state(state);

    Ok(axum::serve(listener, app))
}

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let email_client = EmailClient::new(
            configuration.clone().email_client.base_url,
            sender_email,
            configuration.clone().email_client.authorization_token,
            configuration.email_client.timeout(),
        );
        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))
        .await
        .expect("could not bind port");
        let port = listener.local_addr().unwrap().port();
        info!(
            "Starting app on {:?}:{:?}",
            configuration.application.host, configuration.application.port
        );
        //Start the application
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url.clone(),
        )?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.with_db())
}
