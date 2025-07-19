use std::ops::{Deref, DerefMut};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::RoundedDateTime;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsRequest {
    pub correlation_id: uuid::Uuid,
    pub amount: f64,
    pub requested_at: RoundedDateTime,
}

impl ProcessorPaymentsRequest {
    pub fn new(amount: f64, correlation_id: uuid::Uuid) -> Self {
        Self {
            correlation_id,
            amount,
            requested_at: Utc::now().into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorHealthCheck {
    pub failing: bool,
    pub min_response_time: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProcessorPaymentsIdRequest {
    pub id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcessorPaymentsIdResponse {
    pub inner: ProcessorPaymentsRequest,
}

impl Deref for ProcessorPaymentsIdResponse {
    type Target = ProcessorPaymentsRequest;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ProcessorPaymentsIdResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
