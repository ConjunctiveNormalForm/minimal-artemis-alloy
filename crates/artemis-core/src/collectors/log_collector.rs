use crate::types::{Collector, CollectorStream};
use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider},
    rpc::types::{Filter, Log},
};
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
pub struct LogCollector {
    provider: Arc<DynProvider<AnyNetwork>>,
    filter: Filter,
}

impl LogCollector {
    pub fn new(provider: Arc<DynProvider<AnyNetwork>>, filter: Filter) -> Self {
        Self { provider, filter }
    }
}

/// Implementation of the [Collector](Collector) trait for the [LogCollector](LogCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new logs.
#[async_trait]
impl Collector<Log> for LogCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Log>> {
        let sub = self.provider.subscribe_logs(&self.filter).await?;
        let stream = sub.into_stream();
        Ok(Box::pin(stream))
    }
}
