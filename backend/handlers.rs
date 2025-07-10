use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use backend_core::models::backend::{
    PaymentsRequest, PaymentsResponse, PaymentsSummaryRequest, PaymentsSummaryResponse,
    RequestMetric,
};
use tracing::{Level, instrument};

#[instrument(ret(level = Level::DEBUG))]
pub async fn create_payment(Json(payment): Json<PaymentsRequest>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(PaymentsResponse))
}

#[instrument(ret(level = Level::DEBUG))]
pub async fn get_payment_summary(
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
