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
use futures_util::TryFutureExt;
use redis::{AsyncTypedCommands, ToRedisArgs};
use tokio_retry::{Retry, strategy::FibonacciBackoff};
use tracing::{Level, instrument};

use crate::state::AppState;

const DEFAULT_SET: &str = "default_set";
const FALLBACK_SET: &str = "fallback_set";

enum ServerPayment {
    Default,
    Fallback,
}

#[instrument(skip_all, ret(level = Level::DEBUG))]
pub async fn create_payment(
    State(state): State<AppState>,
    Json(payment): Json<PaymentsRequest>,
) -> impl IntoResponse {
    // TODO: add validation to amount value
    // Call process_payment with retry logic
    tracing::info!(?payment);
    let result = process_payment(state, payment).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(PaymentsResponse)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(PaymentsResponse)),
    }
}

#[instrument(skip_all, err)]
async fn process_payment(state: AppState, payment: PaymentsRequest) -> anyhow::Result<()> {
    let http_client = &state.http_client;
    let mut conn = state
        .redis_client
        .get_multiplexed_tokio_connection()
        .await?;

    let request = ProcessorPaymentsRequest::new(payment.amount, payment.correlation_id);

    // Retry logic for the outbound HTTP request
    let retry_strategy =
        FibonacciBackoff::from_millis(20).max_delay(std::time::Duration::from_millis(100));
    let default_payment_url = state.default_url.join("payments").unwrap();
    let fallback_payment_url = state.fallback_url.join("payments").unwrap();
    let request_ref = &request;
    let timeout = std::time::Duration::from_millis(50);
    let server = Retry::spawn(retry_strategy, || async {
        http_client
            .post(default_payment_url.as_str())
            .json(request_ref)
            .timeout(timeout)
            .send()
            .and_then(|res| {
                async { res.error_for_status() }.and_then(|_| async { Ok(ServerPayment::Default) })
            })
            .or_else(|_| async {
                http_client
                    .post(fallback_payment_url.as_str())
                    .json(request_ref)
                    .timeout(timeout)
                    .send()
                    .await?
                    .error_for_status()
                    .map(|_| ServerPayment::Fallback)
            })
            .await
    })
    .await?;

    // TODO: Serialization overhead here for storing data
    // Maybe use BSON or BINCODE or some other format here instead
    let val = serde_json::to_string(&request)?;

    let set = match server {
        ServerPayment::Default => DEFAULT_SET,
        ServerPayment::Fallback => FALLBACK_SET,
    };

    conn.zadd(set, val, request.requested_at.timestamp_micros())
        .await
        .inspect_err(|err| tracing::error!(?err))?;
    Ok(())
}

#[instrument(skip_all, ret(level = Level::INFO))]
pub async fn get_payment_summary(
    State(state): State<AppState>,
    Query(summary_request): Query<PaymentsSummaryRequest>,
) -> impl IntoResponse {
    tracing::info!(?summary_request);
    let res = get_summaries(state, summary_request).await;
    match res {
        Ok(res) => (StatusCode::OK, Json(res)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get summaries");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PaymentsSummaryResponse::default()),
            )
        }
    }
}

async fn get_summaries(
    state: AppState,
    summary_request: PaymentsSummaryRequest,
) -> anyhow::Result<PaymentsSummaryResponse> {
    let mut conn = state
        .redis_client
        .get_multiplexed_tokio_connection()
        .await?;

    let min = summary_request
        .from
        .map_or(f64::NEG_INFINITY.to_redis_args(), |from| {
            from.timestamp_micros().to_redis_args()
        });
    let max = summary_request
        .to
        .map_or(f64::INFINITY.to_redis_args(), |to| {
            to.timestamp_micros().to_redis_args()
        });

    // TODO: see later if we need to chunk our operations to not have unbounded memory usage from this request
    // let set_count = conn.zcount(DEFAULT_SET, min, max).await?;

    let vals = conn
        .zrangebyscore(DEFAULT_SET, &min, &max)
        .await?
        .into_iter()
        .map(|val| serde_json::from_str::<ProcessorPaymentsRequest>(&val));
    let mut default = RequestMetric::default();
    for val in vals {
        default.total_requests += 1;
        default.total_amount += val?.amount;
    }

    let vals = conn
        .zrangebyscore(FALLBACK_SET, min, max)
        .await?
        .into_iter()
        .map(|val| serde_json::from_str::<ProcessorPaymentsRequest>(&val));
    let mut fallback = RequestMetric::default();
    for val in vals {
        fallback.total_requests += 1;
        fallback.total_amount += val?.amount;
    }

    Ok(PaymentsSummaryResponse { default, fallback })
}
