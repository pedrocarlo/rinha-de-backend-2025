use backend_core::models::{backend::PaymentsRequest, processor::ProcessorPaymentsRequest};
use futures_util::{StreamExt, pin_mut, stream::FuturesUnordered};
use redis::AsyncTypedCommands;
use tokio::select;
use tokio_retry::{Retry, strategy::FibonacciBackoff};

use crate::{
    handlers::{DEFAULT_SET, FALLBACK_SET, ServerPayment},
    state::AppState,
    util::shutdown_signal,
};

pub struct Worker {
    pub channel: async_channel::Receiver<PaymentsRequest>,
    pub state: AppState,
}

impl Worker {
    pub async fn process_payments(self) {
        let channel = self.channel.clone();
        pin_mut!(channel);

        let mut futs = FuturesUnordered::new();

        loop {
            select! {
                Some(payment) = channel.next(), if (futs.is_empty() || futs.len() < 20) => futs.push(self.process_payment(payment)),
                _ = futs.next(), if !futs.is_empty() => {},
                _ = shutdown_signal() => break
            }
        }

        println!("Task shutting down gracefully");
    }

    async fn process_payment(&self, payment: PaymentsRequest) -> anyhow::Result<()> {
        let state = &self.state;
        let http_client = &state.http_client;

        let request = ProcessorPaymentsRequest::new(payment.amount, payment.correlation_id);

        // Retry logic for the outbound HTTP request
        let retry_strategy =
            FibonacciBackoff::from_millis(20).max_delay(std::time::Duration::from_millis(1000));
        let default_payment_url = state.default_url.join("payments").unwrap();
        let fallback_payment_url = state.fallback_url.join("payments").unwrap();
        let request_ref = &request;
        let timeout = std::time::Duration::from_millis(100);
        let server = Retry::spawn(retry_strategy, || async {
            let res_default = http_client
                .post(default_payment_url.as_str())
                .json(request_ref)
                .timeout(timeout)
                .send()
                .await?;

            match res_default.error_for_status() {
                Ok(_) => {
                    return Ok(Some(ServerPayment::Default));
                }
                Err(err) if err.is_timeout() => {}
                Err(_) => {
                    return Ok(None);
                }
            }

            let res_fallback = http_client
                .post(fallback_payment_url.as_str())
                .json(request_ref)
                .timeout(timeout)
                .send()
                .await?;

            match res_fallback.error_for_status() {
                Ok(_) => {
                    return Ok(Some(ServerPayment::Fallback));
                }
                Err(err) if err.is_timeout() => return Err(err),
                Err(_) => {
                    return Ok(None);
                }
            }
        })
        .await?;

        let Some(server) = server else { return Ok(()) };

        // TODO: Serialization overhead here for storing data
        // Maybe use BSON or BINCODE or some other format here instead
        let val = serde_json::to_string(&request)?;

        let set = match server {
            ServerPayment::Default => DEFAULT_SET,
            ServerPayment::Fallback => FALLBACK_SET,
        };

        let mut conn = state
            .redis_client
            .get_multiplexed_tokio_connection()
            .await?;

        conn.zadd(set, val, request.requested_at.timestamp_micros())
            .await
            .inspect_err(|err| tracing::error!(?err))?;
        Ok(())
    }
}
