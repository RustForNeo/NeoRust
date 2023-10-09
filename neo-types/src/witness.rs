use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct Witness {
	pub invocation: String,
	pub verification: String,
}

impl Witness {
	pub fn new(invocation: String, verification: String) -> Self {
		Self { invocation, verification }
	}
}
