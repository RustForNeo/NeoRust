use crate::{
	neo_error::NeoError,
	protocol::{core::neo_trait::NeoTrait, neo_rust::NeoRust},
};
use futures::{Stream, StreamExt, TryStreamExt};
use std::time::Duration;
use tokio::{runtime::Handle, sync::RwLock, time::Interval};

struct BlockIndexActor {
	block_index: RwLock<Option<i32>>,
}

impl BlockIndexActor {
	async fn set_index(&self, index: i32) {
		let mut block_index = self.block_index.write().await;
		*block_index = Some(index);
	}

	async fn get_index(&self) -> Option<i32> {
		self.block_index.read().await.clone()
	}
}

pub struct BlockIndexPolling {
	current_block_index: BlockIndexActor,
}

impl BlockIndexPolling {
	pub async fn block_index_publisher(
		&self,
		polling_interval: i32,
	) -> impl Stream<Item = Result<i32, NeoError>> {
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(polling_interval as u64));

			loop {
				interval.tick().await;
				let latest_index = NeoRust::instance().get_block_count().request().await.unwrap();
				let curr_index = self.current_block_index.get_index().await;

				if curr_index.map(|i| latest_index > i as u32).unwrap_or(true) {
					self.current_block_index.set_index(latest_index as i32).await;
					Ok((curr_index.unwrap_or(0) as u32 + 1..=latest_index).collect::<Vec<_>>())
				} else {
					Ok(vec![])
				}
				.expect("Error getting latest block");

				Err(NeoError::IllegalArgument("Error getting latest block".to_string()))
			}
		})
	}
}
