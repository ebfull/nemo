//! Channels are implementations of `IO` which can be used when building
//! `Session` and designing protocols.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem;
use super::IO;

/// This is an implementation of a blocking channel IO backend. Internally
/// it uses MPSC queues.
pub struct Blocking {
    tx: Sender<Box<usize>>,
    rx: Receiver<Box<usize>>
}

impl Blocking {
    /// Create a new bi-directional channel for protocols.
    pub fn new() -> (Blocking, Blocking) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        (
            Blocking {
                tx: tx1,
                rx: rx2
            },
            Blocking {
                tx: tx2,
                rx: rx1
            }
        )
    }
}

impl<T: Send + 'static> IO<T> for Blocking {
    fn send(&mut self, obj: T) {
        self.tx.send(unsafe { mem::transmute(Box::new(obj)) }).unwrap();
    }

    fn recv(&mut self) -> Option<T> {
        let tmp: Box<usize> = self.rx.recv().unwrap();
        let tmp: Box<T> = unsafe { mem::transmute(tmp) };

        Some(*tmp)
    }

    fn close(&mut self) {
        // we can close the channel now
    }
}
