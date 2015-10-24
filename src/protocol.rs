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
    func: DeferFunc<P, I, (), ()>,
    open: bool,
    _marker: PhantomData<P>
}

impl<P: Protocol, I> Defer<P, I> {
    pub fn new(next: DeferFunc<P, I, (), ()>, open: bool)
               -> Defer<P, I>
    {
        Defer {
            func: next,
            open: open,
            _marker: PhantomData
        }
    }
}

impl<P: Protocol, I> Defer<P, I> {
    pub fn with<'a>(&mut self, io: &'a mut I) -> bool {
        let p: Channel<'a, P, I, (), ()> = Channel(io, PhantomData);

        let new = (self.func)(p);
        self.func = new.func;
        self.open = new.open;

        self.open
    }
}

#[doc(hidden)]
pub type DeferFunc<P, I, E, S> = for<'a> fn(Channel<'a, P, I, E, S>) -> Defer<P, I>;

/// Channels are provided to handlers to act as a "courier" for the session type
/// and a guard for the IO backend.
pub struct Channel<'a, P: Protocol, I: 'a, E: SessionType, S: SessionType>(&'a mut I, PhantomData<(P, E, S)>);

/// `Handler` is implemented on `Protocol` for every session type you expect to defer,
/// including the initial state.
pub trait Handler<I, E: SessionType, S: SessionType>: Protocol + Sized {
    /// Given a channel in a particular state, with a particular environment,
    /// do whatever you'd like with the channel and return `Defer`, which you
    /// can obtain by doing `.defer()` on the channel or `.close()` on the
    /// channel.
    fn with<'a>(Channel<'a, Self, I, E, S>) -> Defer<Self, I>;
}

pub fn channel<'a, P: Protocol, I: IO>(io: &'a mut I) -> Channel<'a, P, I, (), P::Initial> {
    Channel(io, PhantomData)
}

pub fn channel_dual<'a, P: Protocol, I: IO>(io: &'a mut I) -> Channel<'a, P, I, (), <P::Initial as SessionType>::Dual> {
    Channel(io, PhantomData)
}

impl<'a, I, E: SessionType, S: SessionType, P: Handler<I, E, S>> Channel<'a, P, I, E, S> {
    /// Defer the rest of the protocol execution. Useful for returning early.
    /// 
    /// There must be a [`Handler`](trait.Handler.html) implemented for the protocol state you're deferring.
    pub fn defer(self) -> Defer<P, I> {
        let next_func: DeferFunc<P, I, E, S> = Handler::<I, E, S>::with;

        Defer::new(unsafe { mem::transmute(next_func) }, true)
    }
}

impl<'a, I: IO, E: SessionType, P: Protocol> Channel<'a, P, I, E, End> {
    /// Close the channel. Only possible if it's in the `End` state.
    pub fn close(self) -> Defer<P, I> {
        self.0.close();

        let next_func: DeferFunc<P, I, E, End> = Dummy::<P, I, E, End>::with;

        Defer::new(unsafe { mem::transmute(next_func) }, false)
    }
}

impl<'a, I: Transfers<T>, T, E: SessionType, S: SessionType, P: Protocol> Channel<'a, P, I, E, Send<T, S>> {
    /// Send a `T` to IO.
    pub fn send(self, a: T) -> Channel<'a, P, I, E, S> {
        self.0.send(a);

        Channel(self.0, PhantomData)
    }
}

impl<'a, I: Transfers<T>, T, E: SessionType, S: SessionType, P: Protocol> Channel<'a, P, I, E, Recv<T, S>> {
    /// Receive a `T` from IO.
    pub fn recv(self) -> Result<(T, Channel<'a, P, I, E, S>), Self> {
        match self.0.recv() {
            Some(res) => Ok((res, unsafe { mem::transmute(self) })),
            None => {
                Err(self)
            }
        }
    }
}

impl<'a, I, E: SessionType, S: SessionType, P: Protocol> Channel<'a, P, I, E, Nest<S>> {
    /// Enter into a nested protocol.
    pub fn enter(self) -> Channel<'a, P, I, (S, E), S> {
        Channel(self.0, PhantomData)
    }
}

impl<'a, I, N: Peano, E: SessionType + Pop<N>, P: Protocol> Channel<'a, P, I, E, Escape<N>> {
    /// Escape from a nested protocol.
    pub fn pop(self) -> Channel<'a, P, I, E::Tail, E::Head> {
        Channel(self.0, PhantomData)
    }
}

impl<'a, I: Transfers<usize>, E: SessionType, R: SessionType, P: Protocol> Channel<'a, P, I, E, R> {
    /// Select a protocol to advance to.
    pub fn choose<S: SessionType>(self) -> Channel<'a, P, I, E, S> where R: Chooser<S> {
        self.0.send(R::num());

        Channel(self.0, PhantomData)
    }
}

impl<'a, I: Transfers<usize>, // Our IO must be capable of sending a usize
         E: SessionType, // Our current environment
         S: SessionType, // The first branch of our accepting session
         Q: SessionType, // The second branch of our accepting session
         P: Handler<I, E, Accept<S, Q>> // We must be able to handle our current state
            + Acceptor<I, E, Accept<S, Q>> // And we must be able to "accept" with our current state
    > Channel<'a, P, I, E, Accept<S, Q>> {
    /// Accept one of many protocols and advance to its handler.
    pub fn accept(self) -> Defer<P, I> {
        match self.0.recv() {
            Some(num) => {
                <P as Acceptor<I, E, Accept<S, Q>>>::defer(num)
            },
            None => {
                self.defer()
            }
        }
    }
}


struct Dummy<P, I, E, S>(PhantomData<(P, I, E, S)>);
impl<I, P: Protocol, E: SessionType, S: SessionType> Dummy<P, I, E, S> {
    fn with<'a>(_: Channel<'a, P, I, E, S>) -> Defer<P, I> {
        panic!("Channel was closed!");
    }
}