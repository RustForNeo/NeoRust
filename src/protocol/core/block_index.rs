use crate::{
	neo_error::NeoError,
	protocol::{core::neo_trait::NeoTrait, http_service::HttpService, neo_rust::NeoRust},
};
use futures::{Stream, StreamExt, TryStreamExt};
use std::{error::Error, time::Duration};
use tokio::{runtime::Handle, sync::RwLock};

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
		executor: &Handle,
		polling_interval: i32,
	) -> impl Stream<Item = Result<i32, NeoError>> {
		let interval = tokio::time::interval(Duration::from_secs(polling_interval as u64));

		interval
			.map(move |_| {
				let latest_block_index = NeoRust::<HttpService>::instance()
					.get_block_count()
					.execute(executor)
					.map(|res| res.get_result() - 1);

				async move {
					let curr_index = self.current_block_index.get_index().await;

					if let Some(latest_index) = latest_block_index.await.unwrap() {
						if curr_index.map(|i| latest_index > i).unwrap_or(true) {
							self.current_block_index.set_index(latest_index).await;
							Ok((curr_index.unwrap_or(0) + 1..=latest_index).collect::<Vec<_>>())
						} else {
							Ok(None)
						}
					}

					Err(NeoError::IllegalArgument("Error getting latest block".to_string()))
				}
			})
			.try_flatten()
			.filter_map(|x| async move { x })
			.flat_map(|blocks| futures::stream::iter(blocks).map(Ok))
	}
}
