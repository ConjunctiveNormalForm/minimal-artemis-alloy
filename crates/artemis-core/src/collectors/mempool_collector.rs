use async_trait::async_trait;

use alloy::{
    network::{AnyNetwork, AnyTxEnvelope},
    providers::Provider,
    pubsub::PubSubFrontend,
    rpc::types::{serde_helpers::WithOtherFields, Transaction},
};
use std::sync::Arc;
use tracing::error;

use crate::types::{Collector, CollectorStream};
use anyhow::Result;
use tokio_stream::StreamExt;

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
impl<P> Collector<WithOtherFields<Transaction<AnyTxEnvelope>>> for MempoolCollector<P>
where
    P: Provider<PubSubFrontend, AnyNetwork>,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, WithOtherFields<Transaction<AnyTxEnvelope>>>> {
        let sub = match self.provider.subscribe_pending_transactions().await {
            Ok(sub) => sub,
            Err(e) => {
                error!("Error subscribing to pending transactions: {:?}", e);
                return Err(anyhow::anyhow!(
                    "Error subscribing to pending transactions: {:?}",
                    e
                ));
            }
        };
        let stream = sub.into_stream().then(move |hash| async move {
            match self.provider.get_transaction_by_hash(hash).await {
                Ok(Some(tx)) => Some(tx),
                Ok(None) => None,
                Err(e) => {
                    error!("Error getting transaction by hash: {:?}", e);
                    None
                }
            }
        })
        .filter_map(|res| res);

        Ok(Box::pin(stream))
    }
}
