use backend_core::models::{backend::PaymentsRequest, processor::ProcessorPaymentsRequest};
use futures_util::{StreamExt, TryFutureExt, pin_mut, stream::FuturesUnordered};
use redis::AsyncTypedCommands;
use tokio::select;
use tokio_retry::{Retry, strategy::FibonacciBackoff};
use tracing::instrument;

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
                Some(payment) = channel.next(), if (futs.is_empty() || futs.len() < 100) => futs.push(self.process_payment(payment)),
                _ = futs.next(), if !futs.is_empty() => {},
                _ = shutdown_signal() => break
            }
        }

        println!("Task shutting down gracefully");
    }

    #[instrument(skip_all, err)]
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
        let timeout = std::time::Duration::from_millis(50);
        let server = Retry::spawn(retry_strategy, || async {
            http_client
                .post(default_payment_url.as_str())
                .json(request_ref)
                .timeout(timeout)
                .send()
                .and_then(|res| {
                    async { res.error_for_status() }
                        .and_then(|_| async { Ok(ServerPayment::Default) })
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
