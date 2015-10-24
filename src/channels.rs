//! Channels are implementations of `IO` which can be used when building
//! `Session` and designing protocols.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem;
use super::{Transfers, ChannelClaim, IO};

/// This is an implementation of a blocking channel IO backend. Internally
/// it uses MPSC queues.
pub struct Blocking {
    claim: Option<ChannelClaim>,
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
                claim: None,
                tx: tx1,
                rx: rx2
            },
            Blocking {
                claim: None,
                tx: tx2,
                rx: rx1
            }
        )
    }
}

unsafe impl IO for Blocking {
    fn claim(&mut self, claim: ChannelClaim) {
        assert!(self.claim.is_none());

        self.claim = Some(claim);
    }

    fn close(&mut self, claim: ChannelClaim) {
        assert_eq!(Some(claim), self.claim);

        // we can close the channel now
    }
}

unsafe impl<T: Send + 'static> Transfers<T> for Blocking {
    fn send(&mut self, obj: T, claim: ChannelClaim) {
        assert_eq!(Some(claim), self.claim);

        self.tx.send(unsafe { mem::transmute(Box::new(obj)) }).unwrap();
    }

    fn recv(&mut self, claim: ChannelClaim) -> Option<T> {
        assert_eq!(Some(claim), self.claim);

        let tmp: Box<usize> = self.rx.recv().unwrap();
        let tmp: Box<T> = unsafe { mem::transmute(tmp) };

        Some(*tmp)
    }
}
