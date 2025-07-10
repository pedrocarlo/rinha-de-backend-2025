use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
/// Payment requisition to be processed
pub struct PaymentsRequest {
    pub correlation_id: uuid::Uuid,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentsResponse;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PaymentsSummaryRequest {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PaymentsSummaryResponse {
    pub default: RequestMetric,
    pub fallback: RequestMetric,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestMetric {
    pub total_requests: u64,
    pub total_amount: f64,
}
