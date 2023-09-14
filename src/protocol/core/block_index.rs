use std::sync::Mutex;
use std::time::Duration;
use reqwest::Client;
use futures::{StreamExt, stream};
use tokio::time::{interval, Duration};

pub struct BlockIndex {
    index: Mutex<Option<u32>>,
}

impl BlockIndex {
    fn update(&self, index: u32) {
        let mut lock = self.index.lock().unwrap();
        *lock = Some(index);
    }

    fn get(&self) -> Option<u32> {
        let lock = self.index.lock().unwrap();
        lock.clone()
    }
}

pub async fn block_index_stream(
    client: &Client,
    interval: Duration
) -> impl Stream<Item = u32> {

    let index = BlockIndex { index: Mutex::new(None) };

    let mut interval = interval(interval);

    stream::unfold(index, |index| async move {

        let latest_index = client.get_block_count().await? - 1;

        if index.get().is_none() {
            index.update(latest_index);
        }

        if latest_index > index.get().unwrap() {
            let current = index.get().unwrap();
            index.update(latest_index);
            return Poll::Ready(Some((current..latest_index).collect(), index)));
        }

        Poll::Pending
    }).flat_map(|range| stream::iter(range))

}