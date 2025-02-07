use alloy::{
    consensus::Transaction,
    eips::{BlockId, BlockNumberOrTag},
    network::{AnyNetwork, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{serde_helpers::WithOtherFields, BlockTransactionsKind, TransactionRequest},
};
use alloy_node_bindings::{Anvil, AnvilInstance};
use artemis_core::{
    collectors::{block_collector::BlockCollector, mempool_collector::MempoolCollector},
    executors::mempool_executor::{MempoolExecutor, SubmitTxToMempool},
    types::{Collector, Executor}, wrapper::WrappedProvider,
};

use futures::StreamExt;
use std::{sync::Arc, time::Duration};

/// Spawns Anvil and instantiates an Http provider.
pub async fn spawn_anvil() -> (WrappedProvider<AnyNetwork>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint_url());
    let p = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .on_ws(ws)
        .await
        .unwrap();
    let provider = WrappedProvider::new(p);
    (provider, anvil)
}

/// Test that block collector correctly emits blocks.
#[tokio::test]
async fn test_block_collector_sends_blocks() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let block_collector = BlockCollector::new(provider.clone());
    let block_stream = block_collector.get_event_stream().await.unwrap();
    let block_a = block_stream.into_future().await.0.unwrap();
    let block_b = provider
        .get_block(
            BlockId::Number(BlockNumberOrTag::Latest),
            BlockTransactionsKind::Hashes,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(block_a.hash, block_b.header.hash);
}

/// Test that mempool collector correctly emits blocks.
#[tokio::test]
async fn test_mempool_collector_sends_txs() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let mempool_collector = MempoolCollector::new(provider.clone());
    let mempool_stream = mempool_collector
        .get_event_stream()
        .await
        .unwrap_or_else(|e| {
            panic!("Error getting mempool stream: {:?}", e);
        });

    let account = provider.get_accounts().await.unwrap()[0];
    let value = U256::from(42);
    let gas_price = 100000000000000000_u128;
    let tx = TransactionRequest::default()
        .with_from(account)
        .with_to(account)
        .with_value(value)
        .with_gas_price(gas_price);

    let _ = provider
        .send_transaction(WithOtherFields::new(tx))
        .await
        .unwrap();
    let tx = mempool_stream.into_future().await.0.unwrap();
    assert_eq!(tx.value(), value.into());
}

/// Test that the mempool executor correctly sends txs
#[tokio::test]
async fn test_mempool_executor_sends_tx_simple() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let mempool_executor = MempoolExecutor::new(provider.clone());

    let account = provider.get_accounts().await.unwrap()[0];
    let value = U256::from(42);
    let gas_price = 100000000000000000_u128;
    let tx = TransactionRequest::default()
        .with_to(account)
        .with_from(account)
        .with_value(value)
        .with_gas_price(gas_price);
    let action = SubmitTxToMempool {
        tx: WithOtherFields::new(tx),
        gas_bid_info: None,
    };
    mempool_executor.execute(action).await.unwrap();
    //Sleep to seconds so that the tx has time to be mined
    tokio::time::sleep(Duration::from_secs(2)).await;
    let count = provider.get_transaction_count(account).await.unwrap();
    assert_eq!(count, 1);
}
