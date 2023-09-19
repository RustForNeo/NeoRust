use crate::protocol::core::stack_item::StackItem;
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Error, Debug, Display)]
pub enum ProtocolError {
	#[error("RPC responses error: {error}")]
	RpcResponse { error: String },
	#[error("Invocation fault state: {error}")]
	InvocationFaultState { error: String },
	#[error("Client connection error: {message}")]
	ClientConnection { message: String },
	#[error("Cannot cast {item} to {target}")]
	StackItemCast { item: StackItem, target: String },
	#[error("Illegal state: {message}")]
	IllegalState { message: String },
	#[error("HTTP error: {0}")]
	HttpError(#[from] reqwest::Error),
}
