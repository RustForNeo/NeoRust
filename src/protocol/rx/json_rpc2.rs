use crate::{
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, responses::neo_block::NeoBlock},
		neo_rust::NeoRust,
	},
};
use futures::{stream::iter, Stream, StreamExt, TryStreamExt};
use std::time::Duration;
use tokio::{task::spawn_blocking, time::sleep};

#[derive(Debug, Clone)]
pub struct JsonRpc2 {}

impl JsonRpc2 {
	pub fn new() -> Self {
		Self {}
	}

	pub async fn block_index_publisher(
		&mut self,
		polling_interval: i32,
	) -> impl Stream<Item = i32> {
		let mut index = self.latest_block_index_publisher().await.unwrap();

		futures::stream::unfold(index, |last_index| async move {
			sleep(Duration::from_secs(polling_interval as u64)).await;

			let latest_index = self.latest_block_index_publisher().await.unwrap();
			if latest_index > last_index {
				Some((latest_index, latest_index))
			} else {
				None
			}
		})
		.boxed()
	}

	pub async fn block_publisher(
		&mut self,
		full_transaction_objects: bool,
		polling_interval: i32,
	) -> impl Stream<Item = NeoBlock> {
		self.block_index_publisher(polling_interval)
			.await
			.and_then(move |index| {
				let neo_rust = NeoRust::instance().clone();
				let full_transaction_objects = full_transaction_objects;
				async move {
					neo_rust.get_block(index, full_transaction_objects).request().await.unwrap()
				}
			})
			.boxed()
	}

	pub async fn replay_blocks_publisher(
		&self,
		start_block: i32,
		end_block: i32,
		full_transaction_objects: bool,
		ascending: bool,
	) -> impl Stream<Item = NeoBlock> {
		let blocks = if ascending {
			(start_block..=end_block).collect::<Vec<_>>()
		} else {
			(end_block..=start_block).rev().collect::<Vec<_>>()
		};

		let stream = iter(blocks.into_iter().map(move |block| {
			let neo_rust = NeoRust::instance().clone();
			let full_transaction_objects = full_transaction_objects;
			async move {
				neo_rust
					.get_block_by_index(block as u32, full_transaction_objects)
					.request()
					.await
					.unwrap()
			}
		}));
		stream.buffer_unordered(10).collect::<Vec<_>>().await.boxed()
	}

	pub async fn catch_up_to_latest_block_publisher(
		&mut self,
		start_block: i32,
		full_transaction_objects: bool,
		on_caught_up_publisher: impl Stream<Item = Result<NeoBlock, NeoError>>,
	) -> impl Stream<Item = Result<NeoBlock, NeoError>> {
		let latest_block = self.latest_block_index_publisher().await.unwrap();

		if start_block >= latest_block {
			Box::pin(on_caught_up_publisher)
		} else {
			let replay_stream = self
				.replay_blocks_publisher(start_block, latest_block, full_transaction_objects, false)
				.await;

			let new_publisher = self
				.catch_up_to_latest_block_publisher(
					latest_block + 1,
					full_transaction_objects,
					on_caught_up_publisher,
				)
				.await;

			replay_stream.chain(new_publisher)
		}
	}
	pub async fn catch_up_to_latest_and_subscribe_to_new_blocks_publisher(
		&mut self,
		start_block: i32,
		full_transaction_objects: bool,
		polling_interval: i32,
	) -> impl Stream<Item = Result<NeoBlock, NeoError>> {
		self.catch_up_to_latest_block_publisher(
			start_block,
			full_transaction_objects,
			self.block_publisher(full_transaction_objects, polling_interval).await,
		)
		.await
	}
	pub async fn latest_block_index_publisher(&self) -> Result<i32, NeoError> {
		let block_count = spawn_blocking(async move || {
			NeoRust::instance().get_block_count().request().await.unwrap()
		})
		.await
		.unwrap() - 1;

		Ok(block_count as i32)
	}
}
