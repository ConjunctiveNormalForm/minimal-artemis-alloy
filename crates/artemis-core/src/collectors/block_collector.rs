use crate::types::{Collector, CollectorStream};
use alloy::{
    network::AnyNetwork,
    primitives::{BlockHash, BlockNumber},
    providers::{DynProvider, Provider},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;

/// A collector that listens for new blocks, and generates a stream of
/// [events](NewBlock) which contain the block number and hash.
pub struct BlockCollector {
    provider: Arc<DynProvider<AnyNetwork>>,
}

/// A new block event, containing the block number and hash.
#[derive(Debug, Clone)]
pub struct NewBlock {
    pub hash: BlockHash,
    pub number: BlockNumber,
}

impl BlockCollector {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for the [BlockCollector](BlockCollector).
/// To be able to use subscribe* methods, Provider needs to use BoxTransport over PubSubFrontend as transport.
/// See [issue #296](https://github.com/alloy-rs/alloy/issues/296).
#[async_trait]
impl Collector<NewBlock> for BlockCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, NewBlock>> {
        let subscription = self.provider.subscribe_blocks().await?;
        let stream = subscription.into_stream().map(|header| NewBlock {
            hash: header.hash,
            number: header.inner.number,
        });
        Ok(Box::pin(stream))
    }
}
