use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use backend_core::models::{
    backend::{
        PaymentsRequest, PaymentsResponse, PaymentsSummaryRequest, PaymentsSummaryResponse,
        RequestMetric,
    },
    processor::ProcessorPaymentsRequest,
};
use redis::AsyncCommands;
use tokio_retry::{Retry, strategy::ExponentialBackoff};
// use tracing::{Level, instrument};

use crate::state::AppState;

// #[instrument(ret(level = Level::DEBUG))]
pub async fn create_payment(
    State(state): State<AppState>,
    Json(payment): Json<PaymentsRequest>,
) -> impl IntoResponse {
    // TODO: add validation to amount value
    // Call process_payment with retry logic
    let result = process_payment(state, payment).await;
    match result {
        Ok(_) => (StatusCode::CREATED, Json(PaymentsResponse)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to process payment");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(PaymentsResponse))
        }
    }
}

async fn process_payment(state: AppState, payment: PaymentsRequest) -> anyhow::Result<()> {
    let http_client = &state.http_client;
    let mut conn = state
        .redis_client
        .get_multiplexed_tokio_connection()
        .await?;

    let request = ProcessorPaymentsRequest::new(payment.amount, payment.correlation_id);

    // Retry logic for the outbound HTTP request
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .max_delay(std::time::Duration::from_millis(100))
        .take(5);
    let _response = Retry::spawn(retry_strategy, || async {
        http_client
            .post(state.default_url.as_str())
            .json(&request)
            .send()
            .await?
            .error_for_status()
    })
    .await?;

    let _: () = conn.incr("default_total_requests", 1).await?;
    let _: () = conn.incr("default_total_amount", request.amount).await?;

    Ok(())
}

// #[instrument(ret(level = Level::DEBUG))]
pub async fn get_payment_summary(
    State(state): State<AppState>,
    Query(summary_request): Query<PaymentsSummaryRequest>,
) -> impl IntoResponse {
    let res = PaymentsSummaryResponse {
        default: RequestMetric {
            total_requests: 0,
            total_amount: 0.0,
        },
        fallback: RequestMetric {
            total_requests: 0,
            total_amount: 0.0,
        },
    };
    (StatusCode::CREATED, Json(res))
}
