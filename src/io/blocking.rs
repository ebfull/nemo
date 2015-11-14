//! Channels are implementations of `IO` which can be used when building
//! `Session` and designing protocols.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem;
use super::{Transfers, IO};
use super::super::{Channel};
use super::super::session_types::{Alias, SessionType};

/// This is an implementation of a blocking channel IO backend. Internally
/// it uses MPSC queues.
pub struct Blocking {
    tx: Sender<Box<usize>>,
    rx: Receiver<Box<usize>>
}

impl Blocking {
    /// Create a new bi-directional channel for protocols.
    pub fn new<A: Alias>() -> (Channel<Blocking, <A as Alias>::Id>,
                               Channel<Blocking,<<A as Alias>::Id as SessionType>::Dual>) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        (
            Channel::new(Blocking {
                tx: tx1,
                rx: rx2
            }),
            Channel::new(Blocking {
                tx: tx2,
                rx: rx1
            })
        )
    }
}

unsafe impl IO for Blocking {
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

unsafe impl<T: Send + 'static> Transfers<T> for Blocking {
    unsafe fn send(&mut self, obj: T) {
        self.tx.send(mem::transmute(Box::new(obj))).unwrap();
    }

    unsafe fn recv(&mut self) -> Option<T> {
        let tmp: Box<usize> = self.rx.recv().unwrap();
        let tmp: Box<T> = mem::transmute(tmp);

        Some(*tmp)
    }
}
