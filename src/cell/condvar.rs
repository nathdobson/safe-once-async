use futures::poll;
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Formatter};
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::ptr::null;
use std::task::{Context, Poll, Waker};

#[derive(Copy, Clone, Debug)]
enum WaiterState {
    Running,
    Notified,
    Finished,
}

struct Waiter {
    next: Cell<*const Waiter>,
    prev: Cell<*const Waiter>,
    waker: Cell<Option<Waker>>,
    state: Cell<WaiterState>,
}

#[derive(Debug)]
pub struct Condvar {
    front: Cell<*const Waiter>,
    back: Cell<*const Waiter>,
}

#[must_use]
pub struct Guard<'a> {
    condvar: Option<&'a Condvar>,
}

struct Wait<'a, 'w> {
    condvar: &'a Condvar,
    waiter: &'w Waiter,
}

impl<'a, 'w> Unpin for Wait<'a, 'w> {}

impl<'a, 'w> Future for Wait<'a, 'w> {
    type Output = Guard<'a>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = &mut self;
        match this.waiter.state.get() {
            WaiterState::Running => {
                this.waiter.waker.set(Some(cx.waker().clone()));
                Poll::Pending
            }
            WaiterState::Notified => {
                this.waiter.state.set(WaiterState::Finished);
                return Poll::Ready(Guard {
                    condvar: Some(self.condvar),
                });
            }
            WaiterState::Finished => {
                panic!("Already finished")
            }
        }
    }
}

impl<'a, 'w> Drop for Wait<'a, 'w> {
    fn drop(&mut self) {
        unsafe {
            match self.waiter.state.get() {
                WaiterState::Running => self.condvar.remove(self.waiter),
                WaiterState::Notified => self.condvar.notify(),
                WaiterState::Finished => {}
            }
        }
    }
}

impl Condvar {
    pub const fn new() -> Self {
        Condvar {
            front: Cell::new(null()),
            back: Cell::new(null()),
        }
    }
    unsafe fn push(&self, waiter: &Waiter) {
        let old_back = self.back.replace(waiter);
        if old_back != null() {
            (*old_back).next.set(waiter);
            (*waiter).prev.set(old_back);
        }
    }
    unsafe fn remove(&self, waiter: &Waiter) {
        if waiter.prev.get() == null() {
            assert_eq!(self.front.get(), waiter);
            self.front.set(waiter.next.get());
            (*waiter.next.get()).prev.set(null());
        } else {
            (*waiter.prev.get()).next.set(waiter.next.get())
        }
    }
    pub async fn wait(&self) -> Guard {
        unsafe {
            let waiter = Waiter {
                next: Cell::new(null()),
                prev: Cell::new(null()),
                waker: Cell::new(None),
                state: Cell::new(WaiterState::Running),
            };
            self.push(&waiter);
            Wait {
                condvar: &self,
                waiter: &waiter,
            }
            .await
        }
    }
    pub fn notify(&self) {}
}

impl<'a> Guard<'a> {
    pub fn consume(mut self) {
        self.condvar.take();
    }
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        if let Some(condvar) = self.condvar.take() {
            condvar.notify();
        }
    }
}

impl Debug for Waiter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Waiter")
            .field("next", &self.next)
            .field("prev", &self.prev)
            .field("state", &self.state)
            .finish()
    }
}
