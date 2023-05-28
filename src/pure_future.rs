use std::future::Future;
use std::mem;
use std::ops::DerefMut;
use std::panic::resume_unwind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;
use crate::spawned_future::SpawnedFuture;

// A future that has no effect, and therefore can be stalled indefinitely without problems.
pub trait PureFuture: 'static + Send + Future {}

impl<F: 'static + Future + Send> PureFuture for SpawnedFuture<F::Output, F> where F::Output: Send {}

impl<T: 'static + Send> PureFuture for JoinHandle<T> {}

impl<F: PureFuture + Unpin> PureFuture for Box<F> {}

impl<F: 'static + Send + DerefMut> PureFuture for Pin<F> where F::Target: PureFuture {}