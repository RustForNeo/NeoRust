use std::time::Duration;
use serde_json::json;
use tokio::runtime::Handle;
use futures::{stream, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use crate::protocol::core::responses::neo_address::NeoAddress;
use crate::protocol::core::responses::neo_block::NeoBlock;
use crate::types::{Address, H256, U256};
use crate::types::hash256::H256;

#[derive(Deserialize,Serialize,Debug)]
#[serde(tag = "notify_type")]
enum Event {
    Transfer {
        #[serde(rename = "from")]
        from: Address,
        #[serde(rename = "to")]
        to: NeoAddress,
        amount: U256,
        asset: H256,
    },

    Approval {
        owner: Address,
        spender: Address,
        amount: U256,
        asset: H256,
    },
    Mint {
        to: Address,
        amount: U256,
        asset: H256
    },

    Burn {
        from: Address,
        amount: U256,
        asset: H256,
    },
    // Other event types
}

#[derive(Clone, Debug)]
pub struct JsonRpc2 {
    executor: Handle,
}

impl JsonRpc2 {

    pub fn new(executor: Handle) -> Self {
        Self {
            executor
        }
    }

    pub async fn block_index_stream(&self, interval: u64) -> impl Stream<Item = u32> {
        let mut current = self.neo.get_block_count().await;

        stream::unfold(current, |last| async move {
            tokio::time::sleep(Duration::from_secs(interval)).await;

            let latest = self.neo.get_block_count().await;
            if latest > last {
                Some((latest - 1, latest))
            } else {
                None
            }
        })
            .filter_map(|x| async move { x })
    }

    pub async fn block_stream(&self, full_transactions: bool, interval: u64) -> impl Stream<Item = NeoBlock> {
        let index_stream = self.block_index_stream(interval);
        index_stream.map(|index| {
            self.neo.get_block(index, full_transactions)
        }).buffer_unordered(8)
    }

    pub async fn replay_blocks_stream(&self, start: u32, end: u32, full_transactions: bool, ascending: bool) -> impl Stream<Item = NeoBlock> {
        let blocks: Vec<u32> = if ascending {
            (start..=end).collect()
        } else {
            (end..=start).rev().collect()
        };

        stream::iter(blocks)
            .map(|index| self.neo.get_block(index, full_transactions))
            .buffer_unordered(8)
    }

    pub async fn catch_up_to_latest_stream(&self, start: u32, full_transactions: bool) -> impl Stream<Item = NeoBlock> {

        let mut current = start;

        stream::unfold(current, |last| async move {

            let latest = self.neo.get_block_count().await;

            if &last >= &latest {
                None
            } else {
                let next = &last + 1;
                Some((self.neo.get_block(last, full_transactions), next))
            }

        })

    }

    // other methods...
    pub async fn get_account(&self, address: &str) -> Result<AccountState> {
        let params = json!([address]);

        let response: RpcResponse<AccountState> = self
            .neo
            .rpc_call("getaccountstate", params)
            .await?;

        response.result
    }
    pub async fn get_asset(&self, asset_id: &str) -> Result<Asset> {
        let script_hash = asset_id_to_script_hash(asset_id);

        let response: RpcResponse<Asset> = self
            .neo
            .rpc_call("getassetstate", json!([script_hash]))
            .await?;

        response.result
    }
    pub async fn get_contract(&self, script_hash: &str) -> Result<Contract> {
        let params = json!([script_hash]);

        let response: RpcResponse<Contract> = self
            .neo
            .rpc_call("getcontractstate", params)
            .await?;

        response.result
    }
    pub fn subscribe_events(&self, contract_hash: &str) -> EventStream {

        let mut stream = self.neo.subscribe(&["contract_event"], json!([contract_hash]));

        stream
            .filter_map(|msg| {
                serde_json::from_str::<LogEvent>(&msg)
                    .ok()
            })
            .map(|event| {
                // map to Event enum
                match event.notify_type {
                    "transfer" => {
                        Event::Transfer {
                            from: event.from,
                            to: event.to,
                            amount: event.amount,
                            asset: event.asset,
                        }
                    },
                    "approval" => {
                        Event::Approval {
                            owner: event.args[0],
                            spender: event.args[1],
                            amount: event.args[2],
                            asset: event.asset,
                        }
                    },
            })
    }

}