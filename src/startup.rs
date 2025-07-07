use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{
        admin_dashboard, change_password_form, health_check_handler, home, log_out, login,
        login_form, post_change_password, pubslish_newsletters_handler, subscribe_handler,
        subscriptions_confirm_handler,
    },
};
use axum::{
    Router,
    extract::{MatchedPath, Request},
    routing::{get, post},
    serve::Serve,
};
use axum_login::tower_sessions::{Expiry, SessionManagerLayer};
use axum_messages::MessagesManagerLayer;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};
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

pub async fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
    redis_uri: SecretString,
) -> Result<Serve<TcpListener, Router, Router>, anyhow::Error> {
    let state = AppState {
        pg_pool: Arc::new(connection),
        email_client: Arc::new(email_client),
        base_url: Arc::new(ApplicationBaseUrl(base_url)),
    };

    //Redis
    let conf = Config::from_url(redis_uri.expose_secret())?;
    let pool = Pool::new(conf, None, None, None, 6)?;
    let _redis_conn = pool.connect();
    pool.wait_for_connect().await?;

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let app = Router::new()
        .route("/health_check", get(health_check_handler))
        .route("/subscriptions", post(subscribe_handler))
        .route("/subscriptions/confirm", get(subscriptions_confirm_handler))
        .route("/newsletters", post(pubslish_newsletters_handler))
        .route("/", get(home))
        .route("/login", get(login_form).post(login))
        .route("/admin/dashboard", get(admin_dashboard))
        .route(
            "/admin/password",
            get(change_password_form).post(post_change_password),
        )
        .route("/admin/logout", post(log_out))
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
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .with_state(state);

    Ok(axum::serve(listener, app))
}

// async fn shutdown_signal(redis_conn_task_abort_handle: AbortHandle) {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("failed to install Ctrl+C handler");
//     };
//
//     let terminate = async {
//         signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };
//
//     tokio::select! {
//         _ = ctrl_c => { redis_conn_task_abort_handle.abort() },
//         _ = terminate => { redis_conn_task_abort_handle.abort() },
//     }
// }

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, anyhow::Error> {
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
            configuration.redis_uri.clone(),
        )
        .await?;
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
