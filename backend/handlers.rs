use async_channel::Sender;
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
use redis::{AsyncTypedCommands, ToRedisArgs};
use tracing::{Level, instrument};

use crate::state::AppState;

pub const DEFAULT_SET: &str = "default_set";
pub const FALLBACK_SET: &str = "fallback_set";

pub enum ServerPayment {
    Default,
    Fallback,
}

#[instrument(skip_all, ret(level = Level::DEBUG))]
pub async fn create_payment(
    State(sender): State<Sender<PaymentsRequest>>,
    Json(payment): Json<PaymentsRequest>,
) -> impl IntoResponse {
    // tokio::spawn(process_payment(state, payment));
    sender.send(payment).await.unwrap();
    // let result = process_payment(state, payment).await;
    // match result {
    //     Ok(_) => (StatusCode::OK, Json(PaymentsResponse)),
    //     Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(PaymentsResponse)),
    // }
    (StatusCode::OK, Json(PaymentsResponse))
}

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

    let mut conn = state
        .redis_client
        .get_multiplexed_tokio_connection()
        .await?;

    let mut pipe = redis::pipe();
    let (default_vals, fallback_val): (Vec<String>, Vec<String>) = pipe
        .atomic()
        .zrangebyscore(DEFAULT_SET, &min, &max)
        .zrangebyscore(FALLBACK_SET, &min, &max)
        .query_async(&mut conn)
        .await?;

    let mut default = RequestMetric::default();
    let mut fallback = RequestMetric::default();

    // TODO: very inneficient
    let vals = default_vals.into_iter().map(|v| {
        serde_json::from_str::<ProcessorPaymentsRequest>(&v)
            .unwrap()
            .amount
    });
    for val in vals {
        default.total_requests += 1;
        default.total_amount += val;
    }

    let vals = fallback_val.into_iter().map(|v| {
        serde_json::from_str::<ProcessorPaymentsRequest>(&v)
            .unwrap()
            .amount
    });
    for val in vals {
        fallback.total_requests += 1;
        fallback.total_amount += val;
    }

    Ok(PaymentsSummaryResponse { default, fallback })
}

pub async fn purge_payments(State(state): State<AppState>) -> impl IntoResponse {
    let mut conn = state
        .redis_client
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    conn.del(DEFAULT_SET).await.unwrap();
    conn.del(FALLBACK_SET).await.unwrap();
    StatusCode::OK
}
