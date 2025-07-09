use std::ops::{Deref, DerefMut};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsRequest {
    correlation_id: uuid::Uuid,
    amount: f64,
    requested_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsResponse {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorHealthCheck {
    failing: bool,
    min_response_time: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsIdRequest {
    id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcessorPaymentsIdResponse {
    inner: ProcessorPaymentsResponse,
}

impl Deref for ProcessorPaymentsIdResponse {
    type Target = ProcessorPaymentsResponse;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ProcessorPaymentsIdResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
