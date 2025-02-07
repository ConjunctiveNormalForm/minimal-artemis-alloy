use crate::types::{Collector, CollectorStream};
use alloy::{
    network::AnyNetwork,
    primitives::{BlockHash, BlockNumber},
    providers::Provider,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;

/// A collector that listens for new blocks, and generates a stream of
/// [events](NewBlock) which contain the block number and hash.
pub struct BlockCollector<P> {
    provider: Arc<P>,
}

/// A new block event, containing the block number and hash.
#[derive(Debug, Clone)]
pub struct NewBlock {
    pub hash: BlockHash,
    pub number: BlockNumber,
}

impl<P> BlockCollector<P> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for the [BlockCollector](BlockCollector).
/// To be able to use subscribe* methods, Provider needs to use BoxTransport over PubSubFrontend as transport.
/// See [issue #296](https://github.com/alloy-rs/alloy/issues/296).
#[async_trait]
impl<P> Collector<NewBlock> for BlockCollector<P>
where
    //    P: Provider<BoxTransport<PubSubFrontend>, AnyNetwork>,
    P: Provider<AnyNetwork> + Send + Sync,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, NewBlock>> {
        let subscription = self.provider.subscribe_blocks().await?;
        let stream = subscription.into_stream().map(|header| NewBlock {
            hash: header.hash,
            number: header.inner.number,
        });
        Ok(Box::pin(stream))
    }
}
