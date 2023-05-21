#![feature(default_free_fn)]
#![deny(unused_must_use)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![feature(impl_trait_in_assoc_type)]
#![allow(dead_code)]
#![feature(type_alias_impl_trait)]
#![feature(const_async_blocks)]
#![feature(core_intrinsics)]
#![feature(const_heap)]
#![feature(const_nonnull_new)]
#![feature(const_option)]
#![feature(const_ptr_write)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(const_pin)]

//!
//! Alternative implementations of `LazyCell`, `LazyLock`, `OnceCell`, and `OnceLock` from `std`.
//! These implementations are safer than the standard implementations because they typically panic
//! instead of deadlocking.
//!
//! # `sync::LazyLock` and `cell::LazyCell`
//! Lazily initialize a static variable with [sync::LazyLock] (or [cell::LazyCell] for single-threaded code).
//! ```
//! use safe_once::sync::LazyLock;
//! static LAZY: LazyLock<String> = LazyLock::new(||"hello".to_string());
//! assert_eq!(*LAZY, "hello");
//! ```
//!
//! # `sync::OnceLock` and `cell::OnceCell`
//! Gain more control over the initialization behavior with [sync::OnceLock] (or [cell::OnceCell] for single-threaded code).
//!
//! ```
//! use safe_once::sync::OnceLock;
//! static ONCE: OnceLock<String> = OnceLock::new();
//! assert_eq!("hello", ONCE.get_or_init(|| "hello".to_string()));
//! ```
//!
//! Or use the `lock` method to have complete flexibility over initialization:
//! ```
//! use safe_once::once::OnceEntry;
//! use safe_once::sync::{OnceLock, RawOnceLock};
//! static ONCE: OnceLock<String> = OnceLock::new();
//! match ONCE.lock(){
//!     OnceEntry::Occupied(value) => unreachable!(),
//!     OnceEntry::Vacant(lock) => {/*fail to initialize*/}
//! }
//! match ONCE.lock(){
//!     OnceEntry::Occupied(value) => unreachable!(),
//!     OnceEntry::Vacant(lock) => {lock.init("hello".to_string());}
//! }
//! match ONCE.lock(){
//!     OnceEntry::Occupied(value) => assert_eq!(value, "hello"),
//!     OnceEntry::Vacant(lock) => unreachable!()
//! }
//! ```
//!
//! # Deadlock detection
//! If a cycle is detected within a single thread, it triggers a panic instead of a deadlock:
//! ```
//! # use std::panic::catch_unwind;
//! use safe_once::sync::LazyLock;
//! static A: LazyLock<String> = LazyLock::new(||B.to_string());
//! static B: LazyLock<String> = LazyLock::new(||A.to_string());
//! let result = catch_unwind(||{ &*A; });
//! assert_eq!(result.unwrap_err().downcast_ref::<String>().unwrap(),
//!            "called `Result::unwrap()` on an `Err` value: CycleError");
//! ```
//!

pub mod error;
pub mod raw;

use std::cell::UnsafeCell;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::thread::panicking;
use raw::{RawOnce, RawOnceState};
use crate::error::{LockError, PoisonError};

pub mod sync;

pub mod cell;

pub mod lazy;
pub mod once;
pub mod async_lazy;
pub mod const_box;