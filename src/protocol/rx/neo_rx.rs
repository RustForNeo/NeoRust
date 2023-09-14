use std::pin::Pin;
use futures::Stream;
use crate::protocol::core::responses::neo_block::NeoBlock;

pub trait NeoRx {

    fn block_stream(&self, full_transactions: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

    fn replay_blocks_stream(&self, start_block: u32, end_block: u32, full_transactions: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

    fn replay_blocks_stream_ascending(&self, start_block: u32, end_block: u32, full_transactions: bool, ascending: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

    fn catch_up_to_latest_stream(&self, start_block: u32, full_transactions: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

    fn catch_up_and_subscribe_stream(&self, start_block: u32, full_transactions: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

    fn subscribe_stream(&self, full_transactions: bool) -> Pin<Box<dyn Stream<Item = NeoBlock> + Send>>;

}