use std::{
    ops::{Div, Mul},
    sync::Arc,
};

use crate::types::Executor;
use alloy::{
    network::{AnyNetwork, TransactionBuilder},
    primitives::U128,
    providers::{DynProvider, Provider},
    rpc::types::{serde_helpers::WithOtherFields, TransactionRequest},
};
use anyhow::{Context, Result};
use async_trait::async_trait;

/// An executor that sends transactions to the mempool.
pub struct MempoolExecutor {
    client: Arc<DynProvider<AnyNetwork>>,
}

/// Information about the gas bid for a transaction.
#[derive(Debug, Clone)]
pub struct GasBidInfo {
    /// Total profit expected from opportunity
    pub total_profit: U128,

    /// Percentage of bid profit to use for gas
    pub bid_percentage: U128,
}

#[derive(Debug, Clone)]
pub struct SubmitTxToMempool {
    pub tx: WithOtherFields<TransactionRequest>,
    pub gas_bid_info: Option<GasBidInfo>,
}

impl MempoolExecutor {
    pub fn new(client: Arc<DynProvider<AnyNetwork>>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Executor<SubmitTxToMempool> for MempoolExecutor {
    /// Send a transaction to the mempool.
    async fn execute(&self, mut action: SubmitTxToMempool) -> Result<()> {
        let gas_usage = self
            .client
            .estimate_gas(&action.tx)
            .await
            .context("Error estimating gas usage: {}")?;

        let bid_gas_price: U128;
        if let Some(gas_bid_info) = action.gas_bid_info {
            // gas price at which we'd break even, meaning 100% of profit goes to validator
            let breakeven_gas_price: U128 = gas_bid_info.total_profit / U128::from(gas_usage);
            // gas price corresponding to bid percentage
            bid_gas_price = breakeven_gas_price
                .mul(U128::from(gas_bid_info.bid_percentage))
                .div(U128::from(100));
        } else {
            bid_gas_price = U128::from(
                self.client
                    .get_gas_price()
                    .await
                    .context("Error getting gas price: {}")?,
            );
        }
        action.tx.set_gas_price(bid_gas_price.to());
        let _ = self.client.send_transaction(action.tx).await?;
        Ok(())
    }
}
