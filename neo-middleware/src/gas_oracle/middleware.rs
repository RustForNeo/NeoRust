use super::{GasOracle, GasOracleError};
use async_trait::async_trait;
use neo_providers::{Middleware, MiddlewareError as METrait, PendingTransaction};
use neo_types::{transaction::eip2718::TypedTransaction, *};
use thiserror::Error;

/// Middleware used for fetching gas prices over an API instead of `neo_gasPrice`.
#[derive(Debug)]
pub struct GasOracleMiddleware<M, G> {
	inner: M,
	gas_oracle: G,
}

impl<M, G> GasOracleMiddleware<M, G>
where
	M: Middleware,
	G: GasOracle,
{
	pub fn new(inner: M, gas_oracle: G) -> Self {
		Self { inner, gas_oracle }
	}
}

#[derive(Debug, Error)]
pub enum MiddlewareError<M: Middleware> {
	#[error(transparent)]
	GasOracleError(#[from] GasOracleError),

	#[error("{0}")]
	MiddlewareError(M::Error),

	#[error("This gas price oracle only works with Legacy and EIP2930 transactions.")]
	UnsupportedTxType,
}

impl<M: Middleware> METrait for MiddlewareError<M> {
	type Inner = M::Error;

	fn from_err(src: M::Error) -> MiddlewareError<M> {
		MiddlewareError::MiddlewareError(src)
	}

	fn as_inner(&self) -> Option<&Self::Inner> {
		match self {
			MiddlewareError::MiddlewareError(e) => Some(e),
			_ => None,
		}
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, G> Middleware for GasOracleMiddleware<M, G>
where
	M: Middleware,
	G: GasOracle,
{
	type Error = MiddlewareError<M>;
	type Provider = M::Provider;
	type Inner = M;

	// OVERRIDEN METHODS

	fn inner(&self) -> &M {
		&self.inner
	}

	async fn fill_transaction(
		&self,
		tx: &mut TypedTransaction,
		block: Option<BlockId>,
	) -> Result<(), Self::Error> {
		match tx {
			TypedTransaction::Legacy(ref mut tx) =>
				if tx.gas_price.is_none() {
					tx.gas_price = Some(self.get_gas_price().await?);
				},
		};

		self.inner().fill_transaction(tx, block).await.map_err(METrait::from_err)
	}

	async fn get_gas_price(&self) -> Result<U256, Self::Error> {
		Ok(self.gas_oracle.fetch().await?)
	}

	async fn estimate_eip1559_fees(
		&self,
		_: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
	) -> Result<(U256, U256), Self::Error> {
		Ok(self.gas_oracle.estimate_eip1559_fees().await?)
	}

	async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
		&self,
		tx: T,
		block: Option<BlockId>,
	) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
		let mut tx = tx.into();
		self.fill_transaction(&mut tx, block).await?;
		self.inner
			.send_transaction(tx, block)
			.await
			.map_err(MiddlewareError::MiddlewareError)
	}
}
