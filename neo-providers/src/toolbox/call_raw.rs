//! Overrides for the `neo_call` rpc method

use crate::{
	core::{responses::neo_find_states::States, transaction::transaction::Transaction},
	utils,
	utils::PinBoxFut,
	JsonRpcClient, Provider, ProviderError,
};
use neo_types::{block::BlockId, Bytes};
use pin_project::pin_project;
use serde::{ser::SerializeTuple, Serialize};
use std::{
	fmt,
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

/// Provides methods for overriding parameters to the `neo_call` rpc method
pub trait RawCall<'a> {
	/// Sets the block number to execute against
	fn block(self, id: BlockId) -> Self;

	fn state(self, state: &'a States) -> Self;

	/// Maps a closure `f` over the result of `.await`ing this call
	fn map<F>(self, f: F) -> Map<Self, F>
	where
		Self: Sized,
	{
		Map::new(self, f)
	}
}

/// A builder which implements [`RawCall`] methods for overriding `neo_call` parameters.
///
/// `CallBuilder` also implements [`std::future::Future`], so `.await`ing a `CallBuilder` will
/// resolve to the result of executing the `neo_call`.
#[must_use = "call_raw::CallBuilder does nothing unless you `.await` or poll it"]
pub enum CallBuilder<'a, P> {
	/// The primary builder which exposes [`RawCall`] methods.
	Build(Caller<'a, P>),
	/// Used by the [`std::future::Future`] implementation. You are unlikely to encounter this
	/// variant unless you are constructing your own [`RawCall`] wrapper type.
	Wait(PinBoxFut<'a, Bytes>),
}

impl<P: fmt::Debug> fmt::Debug for CallBuilder<'_, P> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Build(call) => f.debug_tuple("Build").field(call).finish(),
			Self::Wait(_) => f.debug_tuple("Wait").field(&"< Future >").finish(),
		}
	}
}

impl<'a, P> CallBuilder<'a, P> {
	/// Instantiate a new call builder based on `tx`
	pub fn new(provider: &'a Provider<P>, tx: &'a Transaction) -> Self {
		Self::Build(Caller::new(provider, tx))
	}

	/// Applies a closure `f` to a `CallBuilder::Build`. Does nothing for `CallBuilder::Wait`.
	pub fn map_input<F>(self, f: F) -> Self
	where
		F: FnOnce(&mut Caller<'a, P>),
	{
		match self {
			Self::Build(mut call) => {
				f(&mut call);
				Self::Build(call)
			},
			wait => wait,
		}
	}

	/// Returns the inner `Caller` from a `CallBuilder::Build`. Panics if the `CallBuilder` future
	/// has already been polled.
	pub fn unwrap(self) -> Caller<'a, P> {
		match self {
			Self::Build(b) => b,
			_ => panic!("CallBuilder::unwrap on a Wait value"),
		}
	}
}

impl<'a, P> RawCall<'a> for CallBuilder<'a, P> {
	/// Sets the block number to execute against
	fn block(self, id: BlockId) -> Self {
		self.map_input(|call| call.input.block = Some(id))
	}
	fn state(self, state: &'a States) -> Self {
		self.map_input(|call| call.input.state = Some(state))
	}
}

impl<'a, P: JsonRpcClient> Future for CallBuilder<'a, P> {
	type Output = Result<Bytes, ProviderError>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let pin = self.get_mut();
		loop {
			match pin {
				CallBuilder::Build(ref call) => {
					let fut = Box::pin(call.execute());
					*pin = CallBuilder::Wait(fut);
				},
				CallBuilder::Wait(ref mut fut) => return fut.as_mut().poll(cx),
			}
		}
	}
}

/// Holds the inputs to the `neo_call` rpc method along with the rpc provider.
/// This type is constructed by [`CallBuilder::new`].
#[derive(Clone, Debug)]
pub struct Caller<'a, P> {
	provider: &'a Provider<P>,
	input: CallInput<'a>,
}

impl<'a, P> Caller<'a, P> {
	/// Instantiate a new `Caller` based on `tx`
	pub fn new(provider: &'a Provider<P>, tx: &'a Transaction) -> Self {
		Self { provider, input: CallInput::new(tx) }
	}
}

impl<'a, P: JsonRpcClient> Caller<'a, P> {
	/// Executes an `neo_call` rpc request with the overriden parameters. Returns a future that
	/// resolves to the result of the request.
	fn execute(&self) -> impl Future<Output = Result<Bytes, ProviderError>> + 'a {
		self.provider.request("neo_call", utils::serialize(&self.input))
	}
}

/// The input parameters to the `neo_call` rpc method
#[derive(Clone, Debug, PartialEq, Eq)]
struct CallInput<'a> {
	tx: &'a Transaction,
	block: Option<BlockId>,
	state: Option<&'a States>,
}

impl<'a> CallInput<'a> {
	fn new(tx: &'a Transaction) -> Self {
		Self { tx, block: None, state: None }
	}
}

impl<'a> Serialize for CallInput<'a> {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		let len = 2 + self.state.is_some() as usize;

		let mut tup = serializer.serialize_tuple(len)?;
		tup.serialize_element(self.tx)?;

		let block = self.block.unwrap();
		tup.serialize_element(&block)?;

		if let Some(state) = self.state {
			tup.serialize_element(state)?;
		}
		tup.end()
	}
}

/// An implementer of [`RawCall`] that maps a function `f` over the output of the inner future.
///
/// This struct is created by the [`map`] method on [`RawCall`].
///
/// [`map`]: RawCall::map
#[must_use = "call_raw::Map does nothing unless you `.await` or poll it"]
#[derive(Clone)]
#[pin_project]
pub struct Map<T, F> {
	#[pin]
	inner: T,
	f: F,
}

impl<T: fmt::Debug, F> fmt::Debug for Map<T, F> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Map").field("inner", &self.inner).finish()
	}
}

impl<T, F> Map<T, F> {
	/// Instantiate a new map
	pub fn new(inner: T, f: F) -> Self {
		Self { inner, f }
	}
}

impl<'a, T, F> RawCall<'a> for Map<T, F>
where
	T: RawCall<'a>,
{
	/// Sets the block number to execute against
	fn block(self, id: BlockId) -> Self {
		Self { inner: self.inner.block(id), f: self.f }
	}

	/// Note that not all client implementations will support this as a parameter.
	fn state(self, state: &'a States) -> Self {
		Self { inner: self.inner.state(state), f: self.f }
	}
}

impl<T, F, Y> Future for Map<T, F>
where
	T: Future,
	F: FnMut(T::Output) -> Y,
{
	type Output = Y;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let pin = self.project();
		let x = futures_util::ready!(pin.inner.poll(cx));
		Poll::Ready((pin.f)(x))
	}
}
