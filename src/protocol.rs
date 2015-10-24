use std::marker::PhantomData;
use std::mem;
use session_types::*;
use peano::{Peano,Pop};
use super::{IO, Transfers};

/// A `Protocol` describes the underlying protocol, including the "initial" session
/// type. `Handler`s are defined over concrete `Protocol`s to implement the behavior
/// of a protocol in a given `SessionType`.
pub trait Protocol {
    type Initial: SessionType;
}

/// Handlers must return `Defer` to indicate to the `Session` how to proceed in
/// the future. `Defer` can be obtained by calling `.defer()` on the channel, or
/// by calling `.close()` when the session is `End`.
pub struct Defer<P: Protocol, I> {
    io: Option<I>,
    func: DeferFunc<P, I, (), ()>,
    open: bool,
    _marker: PhantomData<P>
}

impl<P: Protocol, I> Defer<P, I> {
    pub fn new(io: I, next: DeferFunc<P, I, (), ()>, open: bool)
               -> Defer<P, I>
    {
        Defer {
            io: Some(io),
            func: next,
            open: open,
            _marker: PhantomData
        }
    }
}

impl<P: Protocol, I> Defer<P, I> {
    pub fn with(&mut self) -> bool {
        let p: Channel<P, I, (), ()> = Channel(self.io.take().unwrap(), PhantomData);

        let mut new = (self.func)(p);
        self.func = new.func;
        self.open = new.open;
        self.io = Some(new.io.take().unwrap());

        self.open
    }
}

#[doc(hidden)]
pub type DeferFunc<P, I, E, S> = fn(Channel<P, I, E, S>) -> Defer<P, I>;

/// Channels are provided to handlers to act as a "courier" for the session type
/// and a guard for the IO backend.
pub struct Channel<P: Protocol, I, E: SessionType, S: SessionType>(I, PhantomData<(P, E, S)>);

/// `Handler` is implemented on `Protocol` for every session type you expect to defer,
/// including the initial state.
pub trait Handler<I, E: SessionType, S: SessionType>: Protocol + Sized {
    /// Given a channel in a particular state, with a particular environment,
    /// do whatever you'd like with the channel and return `Defer`, which you
    /// can obtain by doing `.defer()` on the channel or `.close()` on the
    /// channel.
    fn with(Channel<Self, I, E, S>) -> Defer<Self, I>;
}

pub fn channel<P: Protocol, I: IO>(io: I) -> Channel<P, I, (), P::Initial> {
    Channel(io, PhantomData)
}

pub fn channel_dual<P: Protocol, I: IO>(io: I) -> Channel<P, I, (), <P::Initial as SessionType>::Dual> {
    Channel(io, PhantomData)
}

impl<I, E: SessionType, S: SessionType, P: Handler<I, E, S>> Channel<P, I, E, S> {
    /// Defer the rest of the protocol execution. Useful for returning early.
    /// 
    /// There must be a [`Handler`](trait.Handler.html) implemented for the protocol state you're deferring.
    pub fn defer(self) -> Defer<P, I> {
        let next_func: DeferFunc<P, I, E, S> = Handler::<I, E, S>::with;

        Defer::new(self.0, unsafe { mem::transmute(next_func) }, true)
    }
}

impl<I: IO, E: SessionType, P: Protocol> Channel<P, I, E, End> {
    /// Close the channel. Only possible if it's in the `End` state.
    pub fn close(mut self) -> Defer<P, I> {
        self.0.close();

        let next_func: DeferFunc<P, I, E, End> = Dummy::<P, I, E, End>::with;

        Defer::new(self.0, unsafe { mem::transmute(next_func) }, false)
    }
}

impl<I: Transfers<T>, T, E: SessionType, S: SessionType, P: Protocol> Channel<P, I, E, Send<T, S>> {
    /// Send a `T` to IO.
    pub fn send(mut self, a: T) -> Channel<P, I, E, S> {
        self.0.send(a);

        Channel(self.0, PhantomData)
    }
}

impl<I: Transfers<T>, T, E: SessionType, S: SessionType, P: Protocol> Channel<P, I, E, Recv<T, S>> {
    /// Receive a `T` from IO.
    pub fn recv(mut self) -> Result<(T, Channel<P, I, E, S>), Self> {
        match self.0.recv() {
            Some(res) => Ok((res, Channel(self.0, PhantomData))),
            None => {
                Err(self)
            }
        }
    }
}

impl<I, E: SessionType, S: SessionType, P: Protocol> Channel<P, I, E, Nest<S>> {
    /// Enter into a nested protocol.
    pub fn enter(self) -> Channel<P, I, (S, E), S> {
        Channel(self.0, PhantomData)
    }
}

impl<I, N: Peano, E: SessionType + Pop<N>, P: Protocol> Channel<P, I, E, Escape<N>> {
    /// Escape from a nested protocol.
    pub fn pop(self) -> Channel<P, I, E::Tail, E::Head> {
        Channel(self.0, PhantomData)
    }
}

impl<I: Transfers<usize>, E: SessionType, R: SessionType, P: Protocol> Channel<P, I, E, R> {
    /// Select a protocol to advance to.
    pub fn choose<S: SessionType>(mut self) -> Channel<P, I, E, S> where R: Chooser<S> {
        self.0.send(R::num());

        Channel(self.0, PhantomData)
    }
}

impl<I: Transfers<usize>, // Our IO must be capable of sending a usize
         E: SessionType, // Our current environment
         S: SessionType, // The first branch of our accepting session
         Q: SessionType, // The second branch of our accepting session
         P: Handler<I, E, Accept<S, Q>> // We must be able to handle our current state
            + Acceptor<I, E, Accept<S, Q>> // And we must be able to "accept" with our current state
    > Channel<P, I, E, Accept<S, Q>> {
    /// Accept one of many protocols and advance to its handler.
    pub fn accept(mut self) -> Defer<P, I> {
        match self.0.recv() {
            Some(num) => {
                <P as Acceptor<I, E, Accept<S, Q>>>::defer(self.0, num)
            },
            None => {
                self.defer()
            }
        }
    }
}


struct Dummy<P, I, E, S>(PhantomData<(P, I, E, S)>);
impl<I, P: Protocol, E: SessionType, S: SessionType> Dummy<P, I, E, S> {
    fn with(_: Channel<P, I, E, S>) -> Defer<P, I> {
        panic!("Channel was closed!");
    }
}