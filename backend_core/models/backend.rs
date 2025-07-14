use std::ops::{Add, AddAssign};

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
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PaymentsSummaryResponse {
    pub default: RequestMetric,
    pub fallback: RequestMetric,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestMetric {
    pub total_requests: u64,
    pub total_amount: f64,
}

impl Add for RequestMetric {
    type Output = RequestMetric;

    fn add(self, rhs: Self) -> Self::Output {
        RequestMetric {
            total_requests: self.total_requests + rhs.total_requests,
            total_amount: self.total_amount + rhs.total_amount,
        }
    }
}

impl AddAssign for RequestMetric {
    fn add_assign(&mut self, rhs: Self) {
        self.total_requests += rhs.total_requests;
        self.total_amount += rhs.total_amount;
    }
}
