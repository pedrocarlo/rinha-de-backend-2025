use async_channel::{Sender, unbounded};
use axum::{
    Router,
    routing::{get, post},
};
use backend_core::models::backend::PaymentsRequest;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    handlers::{create_payment, get_payment_summary, purge_payments},
    state::AppState,
    util::shutdown_signal,
    worker::Worker,
};

mod handlers;
mod state;
mod util;
mod worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing();

    let state = AppState::new();

    let app = new_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn new_app(state: AppState) -> Router {
    let sender = init_workers(state.clone());

    Router::new()
        .route("/payments-summary", get(get_payment_summary))
        .route("/purge-payments", post(purge_payments))
        .with_state(state)
        .route("/payments", post(create_payment))
        .with_state(sender)
        .layer(TraceLayer::new_for_http())
}

fn init_workers(state: AppState) -> Sender<PaymentsRequest> {
    let (sender, receiver) = unbounded::<PaymentsRequest>();
    let workers = std::iter::repeat_with(|| Worker {
        channel: receiver.clone(),
        state: state.clone(),
    })
    .take(1);
    for worker in workers {
        tokio::spawn(worker.process_payments());
    }
    sender
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
                .with_default_directive("tower_http=debug".parse().unwrap())
                .with_default_directive("axum::rejection=trace".parse().unwrap())
                .from_env_lossy(),
        )
        .init();
    guard
}

// TODO: test validation logic here
#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use backend_core::models::backend::PaymentsRequest;
    use serde_json::json;
    use uuid::Uuid;

    use crate::{new_app, state::AppState};

    fn new_test_app() -> TestServer {
        let state = AppState::new();
        let app = new_app(state);
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
