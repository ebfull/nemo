//! Channels are implementations of `IO` which can be used when building
//! `Session` and designing protocols.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem;
use super::{Channel, Protocol, Transfers, IO};
use super::session_types::SessionType;

/// This is an implementation of a blocking channel IO backend. Internally
/// it uses MPSC queues.
pub struct Blocking {
    tx: Sender<Box<usize>>,
    rx: Receiver<Box<usize>>
}

impl Blocking {
    /// Create a new bi-directional channel for protocols.
    pub fn new<P: Protocol>(a: P, b: P) -> (super::Channel<P, Blocking, (), P::Initial>, super::Channel<P, Blocking, (), <P::Initial as SessionType>::Dual>) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        (
            super::channel(Blocking {
                tx: tx1,
                rx: rx2
            }, a),
            super::channel_dual(Blocking {
                tx: tx2,
                rx: rx1
            }, b)
        )
    }
}

unsafe impl IO for Blocking {
    unsafe fn close(&mut self) {
        // we can close the channel now
    }

    /// Send a variable length integer over the channel.
    unsafe fn send_varint(&mut self, num: usize) {
        self.send(num)
    }

    /// Receive a variable length integer from the channel.
    unsafe fn recv_varint(&mut self) -> Option<usize> {
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
