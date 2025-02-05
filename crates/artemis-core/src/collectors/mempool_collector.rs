use async_trait::async_trait;

use alloy::{providers::Provider, rpc::types::Transaction};
use std::sync::Arc;

use crate::types::{Collector, CollectorStream};
use anyhow::Result;

/// A collector that listens for new transactions in the mempool, and generates a stream of
/// [events](Transaction) which contain the transaction.
pub struct MempoolCollector<P> {
    provider: Arc<P>,
}

impl<P> MempoolCollector<P> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for the [MempoolCollector](MempoolCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new transactions.
#[async_trait]
impl<P> Collector<Transaction> for MempoolCollector<P>
where
    P: Provider,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Transaction>> {
        let sub = self.provider.subscribe_full_pending_transactions().await?;
        let stream = sub.into_stream();
        Ok(Box::pin(stream))
    }
}
