pub struct NeoConstants {}
impl NeoConstants {
	// Accounts, Addresses, Keys
	pub const MAX_PUBLIC_KEYS_PER_MULTI_SIG: u32 = 1024;
	pub const HASH160_SIZE: u32 = 20;
	pub const HASH256_SIZE: u32 = 32;
	pub const PRIVATE_KEY_SIZE: u32 = 32;
	pub const PUBLIC_KEY_SIZE_COMPRESSED: u32 = 33;
	pub const SIGNATURE_SIZE: u32 = 64;
	pub const VERIFICATION_SCRIPT_SIZE: u32 = 40;
	pub const MAX_ITERATOR_ITEMS_DEFAULT: u32 = 100;

	pub const MAX_SUBITEMS: u32 = 16;
	pub const MAX_NESTING_DEPTH: u8 = 2;

	// Transactions & Contracts
	pub const CURRENT_TX_VERSION: u8 = 0;
	pub const MAX_TRANSACTION_SIZE: u32 = 102400;
	pub const MAX_TRANSACTION_ATTRIBUTES: u32 = 16;
	pub const MAX_SIGNER_SUBITEMS: u32 = 16;
	pub const MAX_MANIFEST_SIZE: u32 = 0xFFFF;

	pub fn new() -> Self {
		Self {}
	}
}
