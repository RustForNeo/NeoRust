use crate::protocol::core::stack_item::StackItem;

#[derive(Debug)]
pub enum ProtocolError {
	RpcResponse { error: String },
	InvocationFaultState { error: String },
	ClientConnection { message: String },
	StackItemCast { item: StackItem, target: String },
	IllegalState { message: String },
	HttpError(reqwest::Error),
}

impl std::fmt::Display for ProtocolError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			ProtocolError::RpcResponse { error } => {
				write!(f, "RPC responses error: {}", error)
			},
			ProtocolError::InvocationFaultState { error } => {
				write!(f, "Invocation fault state: {}", error)
			},
			ProtocolError::ClientConnection { message } => {
				write!(f, "Client connection error: {}", message)
			},
			ProtocolError::StackItemCast { item, target } => {
				write!(f, "Cannot cast {} to {}", item.type_name(), target)
			},
			ProtocolError::IllegalState { message } => {
				write!(f, "Illegal state: {}", message)
			},
			ProtocolError::HttpError(e) => {
				write!(f, "HTTP error: {}", e)
			},
		}
	}
}

impl std::error::Error for ProtocolError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			ProtocolError::RpcResponse { .. } => None,
			ProtocolError::InvocationFaultState { .. } => None,
			ProtocolError::ClientConnection { .. } => None,
			ProtocolError::StackItemCast { .. } => None,
			ProtocolError::IllegalState { .. } => None,
			ProtocolError::HttpError(e) => Some(e),
		}
	}
	fn description(&self) -> &str {
		match self {
			ProtocolError::RpcResponse { .. } => "RPC responses error",
			ProtocolError::InvocationFaultState { .. } => "Invocation fault state",
			ProtocolError::ClientConnection { .. } => "Client connection error",
			ProtocolError::StackItemCast { .. } => "Cannot cast stack item",
			ProtocolError::IllegalState { .. } => "Illegal state",
			ProtocolError::HttpError(e) => e.to_string(),
		}
	}
	fn cause(&self) -> Option<&dyn std::error::Error> {
		match self {
			ProtocolError::RpcResponse { .. } => None,
			ProtocolError::InvocationFaultState { .. } => None,
			ProtocolError::ClientConnection { .. } => None,
			ProtocolError::StackItemCast { .. } => None,
			ProtocolError::IllegalState { .. } => None,
			ProtocolError::HttpError(e) => Some(e),
		}
	}
}
