use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::mut_cell::MutCell;

pub enum OptionThunk<T, F> {
    Uninit,
    Future(F),
    Value(T),
}

impl<F: Future + Unpin> OptionThunk<F::Output, F> {
    pub const fn new() -> Self {
        OptionThunk::Uninit
    }
    pub const fn new_value(x: F::Output) -> Self {
        OptionThunk::Value(x)
    }
    pub const fn new_future(x: F) -> Self {
        OptionThunk::Future(x)
    }
    pub fn start(&mut self, f: F) {
        match self {
            OptionThunk::Uninit => *self = OptionThunk::Future(f),
            OptionThunk::Future(ref mut f) => unreachable!(),
            OptionThunk::Value(x) => unreachable!(),
        }
    }
    pub fn started(&self) -> bool {
        match self {
            OptionThunk::Uninit => false,
            OptionThunk::Future(_) => true,
            OptionThunk::Value(x) => true,
        }
    }
    // pub async fn get_or_init(&mut self, f: impl FnOnce() -> F) -> &mut F::Output {
    //     match self {
    //         OptionThunk::Uninit => *self = OptionThunk::Future(f()),
    //         OptionThunk::Future(ref mut f) => {}
    //         OptionThunk::Value(x) => return x,
    //     }
    //     self.force().await
    // }
    pub async fn force(&mut self) -> &mut F::Output {
        match self {
            OptionThunk::Uninit => unreachable!(),
            OptionThunk::Future(ref mut f) => {
                let output = Pin::new(f).await;
                *self = OptionThunk::Value(output);
            }
            OptionThunk::Value(x) => return x,
        }
        match self {
            OptionThunk::Uninit => unreachable!(),
            OptionThunk::Future(_) => unreachable!(),
            OptionThunk::Value(x) => return x,
        }
    }
    pub fn get(&self) -> Option<&F::Output> {
        match self {
            OptionThunk::Future(_) => None,
            OptionThunk::Value(x) => Some(x),
            OptionThunk::Uninit => None
        }
    }
}

pub enum Thunk<T, F> {
    Future(MutCell<F>),
    Value(T),
}

impl<F: Future + Unpin> Thunk<F::Output, F> {
    pub const fn new(x: F) -> Self {
        Thunk::Future(MutCell::new(x))
    }
    pub const fn new_value(x: F::Output) -> Self {
        Thunk::Value(x)
    }
    pub async fn get_or_init(&mut self) -> &mut F::Output {
        match self {
            Thunk::Future(ref mut f) => {
                let output = Pin::new(f).await;
                *self = Thunk::Value(output);
            }
            Thunk::Value(x) => return x,
        }
        match self {
            Thunk::Future(_) => unreachable!(),
            Thunk::Value(x) => return x,
        }
    }
    pub fn get(&self) -> Option<&F::Output> {
        match self {
            Thunk::Future(_) => None,
            Thunk::Value(x) => Some(x),
        }
    }
}