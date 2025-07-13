use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    handlers::{create_payment, get_payment_summary},
    state::AppState,
};

mod handlers;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing();

    let app = new_app();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn new_app() -> Router {
    Router::new()
        .route("/payments", post(create_payment))
        .route("/payments-summary", get(get_payment_summary))
        .with_state(AppState::new())
        .layer(TraceLayer::new_for_http())
}

fn init_tracing() -> WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_line_number(true)
                .with_thread_ids(false)
                .with_ansi(true),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive("tower_http=debug".parse().unwrap())
                .add_directive("axum::rejection=trace".parse().unwrap()),
        )
        .init();
    guard
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal;

        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// TODO: test validation logic here
#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use backend_core::models::backend::PaymentsRequest;
    use serde_json::json;
    use uuid::Uuid;

    use crate::new_app;

    fn new_test_app() -> TestServer {
        let app = new_app();
        TestServer::builder().mock_transport().build(app).unwrap()
    }

    #[tokio::test]
    async fn smoke_test_payments() {
        let server = new_test_app();

        let request = PaymentsRequest {
            correlation_id: Uuid::new_v4(),
            amount: 1000.00,
        };

        let response = server
            .post("/payments")
            .json(&request)
            .expect_success()
            .await;
        assert_eq!(response.status_code(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn smoke_test_payments_fail() {
        let server = new_test_app();

        let response = server
            .post("/payments")
            .json(&json!(""))
            .expect_failure()
            .await;
        response.assert_status_failure();
    }
}
