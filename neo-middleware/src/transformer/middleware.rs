use super::{Transformer, TransformerError};
use async_trait::async_trait;
use neo_providers::{
	core::transaction::transaction::Transaction, FilterWatcher, Middleware, MiddlewareError,
	PendingTransaction, PubsubClient, SubscriptionStream,
};
use neo_types::{block::BlockId, filter::Filter, log::Log};
use std::{future::Future, pin::Pin};
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for intercepting transaction requests and transforming them to be executed by
/// the underneath `Transformer` instance.
pub struct TransformerMiddleware<M, T> {
	inner: M,
	transformer: T,
}

impl<M, T> TransformerMiddleware<M, T>
where
	M: Middleware,
	T: Transformer,
{
	/// Creates a new TransformerMiddleware that intercepts transactions, modifying them to be sent
	/// through the Transformer.
	pub fn new(inner: M, transformer: T) -> Self {
		Self { inner, transformer }
	}
}

#[derive(Error, Debug)]
pub enum TransformerMiddlewareError<M: Middleware> {
	#[error(transparent)]
	TransformerError(#[from] TransformerError),

	#[error("{0}")]
	MiddlewareError(M::Error),
}

impl<M: Middleware> MiddlewareError for TransformerMiddlewareError<M> {
	type Inner = M::Error;

	fn from_err(src: M::Error) -> Self {
		TransformerMiddlewareError::MiddlewareError(src)
	}

	fn as_inner(&self) -> Option<&Self::Inner> {
		match self {
			TransformerMiddlewareError::MiddlewareError(e) => Some(e),
			_ => None,
		}
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, T> Middleware for TransformerMiddleware<M, T>
where
	M: Middleware,
	T: Transformer,
{
	type Error = TransformerMiddlewareError<M>;
	type Provider = M::Provider;
	type Inner = M;

	fn inner(&self) -> &M {
		&self.inner
	}

	async fn send_transaction<Tx: Into<Transaction> + Send + Sync>(
		&self,
		tx: Tx,
		block: Option<BlockId>,
	) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
		let mut tx = tx.into();

		// construct the appropriate proxy tx.
		self.transformer.transform(&mut tx)?;

		self.fill_transaction(&mut tx, block).await?;
		// send the proxy tx.
		self.inner
			.send_transaction(tx, block)
			.await
			.map_err(TransformerMiddlewareError::MiddlewareError)
	}

	fn watch<'a, 'life0, 'async_trait>(
		&'a self,
		filter: &'life0 Filter,
	) -> Pin<
		Box<
			dyn Future<Output = Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error>>
				+ Send
				+ 'async_trait,
		>,
	>
	where
		'a: 'async_trait,
	{
		todo!()
	}

	fn subscribe_logs<'a, 'life0, 'async_trait>(
		&'a self,
		filter: &'life0 Filter,
	) -> Pin<
		Box<
			dyn Future<Output = Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>>
				+ Send
				+ 'async_trait,
		>,
	>
	where
		<Self as Middleware>::Provider: PubsubClient,
		'a: 'async_trait,
	{
		todo!()
	}
}
