pub trait AddressExtension {
	fn to_script_hash(&self) -> Result<Vec<u8>, &'static str>;
}

impl AddressExtension for String {
	fn to_script_hash(&self) -> Result<Vec<u8>, &'static str> {
		todo!()
	}
}
