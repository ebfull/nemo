//! Channels are implementations of `IO` which can be used when building
//! `Session` and designing protocols.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem;
use super::{Transfers, IO};
use super::super::{Channel};
use super::super::session_types::{Alias, SessionType};

/// This is an implementation of a Nonblocking channel IO backend. Internally
/// it uses MPSC queues.
pub struct NonBlocking {
    tx: Sender<Box<usize>>,
    rx: Receiver<Box<usize>>
}

impl NonBlocking {
    /// Create a new bi-directional channel for protocols.
    pub fn new<A: Alias>() -> (Channel<NonBlocking, <A as Alias>::Id>,
                               Channel<NonBlocking, <<A as Alias>::Id as SessionType>::Dual>) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        (
            Channel::new(NonBlocking {
                tx: tx1,
                rx: rx2
            }),
            Channel::new(NonBlocking {
                tx: tx2,
                rx: rx1
            })
        )
    }
}

unsafe impl IO for NonBlocking {
    unsafe fn close(&mut self) {
        // we can close the channel now
    }

    /// Send a variable length integer over the channel.
    unsafe fn send_discriminant(&mut self, num: usize) {
        self.send(num)
    }

    /// Receive a variable length integer from the channel.
    unsafe fn recv_discriminant(&mut self) -> Option<usize> {
        self.recv()
    }
}

unsafe impl<T: Send + 'static> Transfers<T> for NonBlocking {
    unsafe fn send(&mut self, obj: T) {
        self.tx.send(mem::transmute(Box::new(obj))).unwrap();
    }

    unsafe fn recv(&mut self) -> Option<T> {
        match self.rx.try_recv() {
            Ok(tmp) => {
                let tmp: Box<T> = mem::transmute(tmp);
                Some(*tmp)
            },
            Err(_) => None
        }
    }
}
