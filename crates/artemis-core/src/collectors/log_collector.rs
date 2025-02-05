use crate::types::{Collector, CollectorStream};
use alloy::{network::AnyNetwork, providers::Provider, pubsub::PubSubFrontend, rpc::types::{Filter, Log}};
use anyhow::Result;
use async_trait::async_trait;
//use ethers::{
//    prelude::Middleware,
//    providers::PubsubClient,
//    types::{Filter, Log},
//};

use std::sync::Arc;

/// A collector that listens for new blockchain event logs based on a [Filter](Filter),
/// and generates a stream of [events](Log).
pub struct LogCollector<P> {
    provider: Arc<P>,
    filter: Filter,
}

impl<P> LogCollector<P> {
    pub fn new(provider: Arc<P>, filter: Filter) -> Self {
        Self { provider, filter }
    }
}

/// Implementation of the [Collector](Collector) trait for the [LogCollector](LogCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new logs.
#[async_trait]
impl<P> Collector<Log> for LogCollector<P>
where
    P: Provider<PubSubFrontend, AnyNetwork>,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Log>> {
        let sub = self.provider.subscribe_logs(&self.filter).await?;
        let stream = sub.into_stream();
        Ok(Box::pin(stream))
    }
}
