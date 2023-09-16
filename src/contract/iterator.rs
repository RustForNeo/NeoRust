// iterator

use serde::{Deserialize, Serialize};
use crate::protocol::core::stack_item::StackItem;
use crate::protocol::neo_rust::NeoRust;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Iterator<T> {
    session_id: String,
    iterator_id: String,
    mapper: fn(StackItem) -> T,
}

impl<T> Iterator<T> {

    pub fn new(session_id: String, iterator_id: String, mapper: fn(StackItem) -> T) -> Self {
        Self {
            session_id,
            iterator_id,
            mapper,
        }
    }

    pub async fn next(&self, count: usize) -> Vec<T> {
        let items = NeoRust::instance().traverse_iter(
            self.session_id.clone(),
            self.iterator_id.clone(),
            count,
        ).await?;

        items.into_iter()
            .map(|item| (self.mapper)(item))
            .collect()
    }

    pub async fn close(&self) -> Result<(), Err> {
        NeoRust::instance().terminate_session(&self.session_id).await
    }
}