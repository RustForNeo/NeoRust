// iterator

use crate::{
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, stack_item::StackItem},
		neo_rust::NeoRust,
	},
	transaction::signer::Signer,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoIterator<T> {
	session_id: String,
	iterator_id: String,
	mapper: fn(StackItem) -> T,
}

impl<T> NeoIterator<T> {
	pub fn new(session_id: String, iterator_id: String, mapper: fn(StackItem) -> T) -> Self {
		Self { session_id, iterator_id, mapper }
	}

	pub async fn next(&self, count: usize) -> Vec<Box<dyn Signer>> {
		let items = NeoRust::instance()
			.traverse_iter(self.session_id.clone(), self.iterator_id.clone(), count)
			.await?;

		items.into_iter().map(|item| (self.mapper)(item)).collect()
	}

	pub async fn close(&self) -> Result<bool, NeoError> {
		NeoRust::instance().terminate_session(&self.session_id).await.request()
	}
}
