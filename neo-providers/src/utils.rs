use crate::ProviderError;
use futures_timer::Delay;
use futures_util::{stream, FutureExt, StreamExt};

use primitive_types::U256;
use serde::Serialize;
use std::{future::Future, pin::Pin};

/// A simple gas escalation policy
pub type EscalationPolicy = Box<dyn Fn(U256, usize) -> U256 + Send + Sync>;

// Helper type alias
#[cfg(target_arch = "wasm32")]
pub(crate) type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = Result<T, ProviderError>> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type PinBoxFut<'a, T> =
	Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

/// Calls the future if `item` is None, otherwise returns a `futures::ok`
pub async fn maybe<F, T, E>(item: Option<T>, f: F) -> Result<T, E>
where
	F: Future<Output = Result<T, E>>,
{
	if let Some(item) = item {
		futures_util::future::ok(item).await
	} else {
		f.await
	}
}

/// Create a stream that emits items at a fixed interval. Used for rate control
pub fn interval(
	duration: instant::Duration,
) -> impl futures_core::stream::Stream<Item = ()> + Send + Unpin {
	stream::unfold((), move |_| Delay::new(duration).map(|_| Some(((), ())))).map(drop)
}

// A generic function to serialize any data structure that implements Serialize trait
pub fn serialize<T: serde::Serialize>(t: &T) -> serde_json::Value {
	serde_json::to_value(t).expect("Failed to serialize value")
}
