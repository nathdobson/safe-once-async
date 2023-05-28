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
#![feature(const_ptr_is_null)]


pub mod raw;

use std::cell::UnsafeCell;
use std::fmt::{Debug, Display, Formatter};
use std::future::poll_fn;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use std::thread::panicking;
use std::future::Future;
use std::mem;

pub mod sync;

pub mod cell;

pub mod async_once;
pub mod async_lazy;
mod const_box;
// mod pure_future;
mod thunk;
pub mod async_fused;
pub mod detached;
mod mut_cell;
pub mod async_static;

struct Droppy {}

impl Future for Droppy {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Poll");
        return Poll::Pending;
    }
}

impl Drop for Droppy {
    fn drop(&mut self) {
        println!("Drop");
    }
}
