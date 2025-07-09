use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
/// Payment requisition to be processed
pub struct PaymentsRequest {
    correlation_id: uuid::Uuid,
    amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentsResponse;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PaymentsSummaryRequest {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PaymentsSummaryResponse {
    default: RequestMetric,
    fallback: RequestMetric,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestMetric {
    total_requests: u64,
    total_amount: f64,
}
