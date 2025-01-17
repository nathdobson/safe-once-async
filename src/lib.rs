#![deny(unused_must_use)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![feature(impl_trait_in_assoc_type)]
#![allow(dead_code)]
#![feature(type_alias_impl_trait)]
#![feature(const_async_blocks)]
#![feature(const_heap)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(exclusive_wrapper)]
#![allow(unused_mut)]
#![feature(trait_alias)]
#![feature(core_intrinsics)]
#![allow(internal_features)]

//!
//! ```
//! use std::future::Future;
//! use std::pin::Pin;
//! use safe_once_async::detached::DetachedFuture;
//! use safe_once_async::sync::AsyncLazyLock;
//! fn init(x:&AsyncLazyLock<Pin<Box<dyn Send+DetachedFuture<Output=usize>>>>) -> impl Send+Future<Output=&usize>{
//!     x.get()
//! }
//! ```
//!
//! ```compile_fail
//! use std::future::Future;
//! use safe_once_async::cell::AsyncLazyCell;
//! use safe_once_async::detached::JoinTransparent;
//! fn init(x:&AsyncLazyCell<JoinTransparent<usize>>) -> impl Send+Future<Output=&usize>{
//!     x.get()
//! }
//! ```
//!
//! ```
//! use std::future::Future;
//! use std::pin::Pin;
//! use safe_once_async::async_lazy::AsyncLazy;
//! use safe_once_async::detached::DetachedFuture;
//! use safe_once_async::raw::AsyncRawFused;
//! use safe_once_async::raw::AsyncRawFusedSync;
//! fn init<R:AsyncRawFusedSync>(x:&AsyncLazy<R,Pin<Box<dyn Send+DetachedFuture<Output=usize>>>>) -> impl Send+Future<Output=&usize>{
//!     x.get()
//! }
//! ```
//!

pub mod raw;

use std::cell::UnsafeCell;
use std::fmt::{Debug, Display, Formatter};
use std::future::poll_fn;
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use std::thread::panicking;

pub mod sync;

pub mod cell;

pub mod async_fused;
pub mod async_lazy;
pub mod async_once;
// pub mod async_static;
// pub mod const_box;
pub mod detached;
mod thunk;
