use std::error::Error;

#[derive(Debug)]
pub enum WalletError {
	AccountState(String),
	NoDefaultAccount,
	NoKeyPair,
}

impl std::fmt::Display for WalletError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			WalletError::AccountState(msg) => {
				write!(f, "Account state error: {}", msg)
			},
			WalletError::NoDefaultAccount => {
				write!(f, "No default account")
			},
			WalletError::NoKeyPair => {
				write!(f, "No key pair")
			},
		}
	}
}

impl std::error::Error for WalletError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		// WalletErrors do not wrap other errors
		None
	}

	fn description(&self) -> &str {
		// Use the Display implementation to generate description
		match self {
			WalletError::AccountState(msg) => msg,
			WalletError::NoDefaultAccount => "No default account",
			WalletError::NoKeyPair => "No key pair",
		}
	}

	fn cause(&self) -> Option<&dyn Error> {
		// The underlying cause is ourselves
		Some(self)
	}
}
