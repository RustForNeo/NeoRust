// iterator

use crate::{
	neo_error::NeoError,
	protocol::{
		core::{neo_trait::NeoTrait, stack_item::StackItem},
		neo_rust::NeoRust,
	},
	transaction::signer::Signer,
};

use crate::protocol::http_service::HttpService;

#[derive(Debug)]
pub struct NeoIterator<T> {
	session_id: String,
	iterator_id: String,
	mapper: fn(StackItem) -> T,
}

// pub struct Iterator<T> {
// 	neo_swift: NeoSwift,
// 	session_id: String,
// 	iterator_id: String,
// 	mapper: Box<dyn Fn(StackItem) -> Result<T, Error> + Send>,
// }

impl<T> NeoIterator<T> {
	pub fn new(session_id: String, iterator_id: String, mapper: fn(StackItem) -> T) -> Self {
		Self { session_id, iterator_id, mapper }
	}

	pub async fn traverse(&self, count: i32) -> Result<Vec<T>, NeoError> {
		let result = NeoRust::<HttpService>::instance()
			.traverse_iterator(self.session_id.clone(), self.iterator_id.clone(), count)
			.request()
			.await?;
		let mapped = result.iter().map(|item| (self.mapper)(item.clone())).collect();
		Ok(mapped)
	}

	pub async fn terminate_session(&self) -> Result<(), NeoError> {
		NeoRust::<HttpService>::instance()
			.terminate_session(self.session_id.clone())
			.request()
			.await
			.expect("Could not terminate session");
		Ok(())
	}
}
