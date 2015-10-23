use std::marker::PhantomData;
use std::mem;
use session_types::*;
use peano::{Peano,Pop};
use super::IO;

/// A `Protocol` describes the underlying protocol, including the "initial" session
/// type. `Handler`s are defined over concrete `Protocol`s to implement the behavior
/// of a protocol in a given `SessionType`.
pub trait Protocol {
    type Initial: SessionType;
}

/// `Session`s are containers of channels which store the current session type
/// internally through a function pointer to a concrete handler.
pub struct Session<P: Protocol, I>(Defer<P, I>);

/// Handlers must return `Defer` to indicate to the `Session` how to proceed in
/// the future. `Defer` can be obtained by calling `.defer()` on the channel, or
/// by calling `.close()` when the session is `Eps`.
pub struct Defer<P: Protocol, I>(pub DeferFunc<I, P, (), ()>, pub PhantomData<P>, pub bool);

#[doc(hidden)]
pub type DeferFunc<I, P, E, S> = for<'a> fn(Chan<'a, I, E, S>) -> Defer<P, I>;

/// Channels are provided to handlers to act as a "courier" for the session type
/// and a guard for the IO backend.
pub struct Chan<'a, I: 'a, E: SessionType, S: SessionType>(&'a mut I, PhantomData<(E, S)>);

/// `Handler` is implemented on `Protocol` for every session type you expect to defer,
/// including the initial state.
pub trait Handler<I, E: SessionType, S: SessionType>: Protocol + Sized {
    /// Given a channel in a particular state, with a particular environment,
    /// do whatever you'd like with the channel and return `Defer`, which you
    /// can obtain by doing `.defer()` on the channel or `.close()` on the
    /// channel.
    fn with<'a>(Chan<'a, I, E, S>) -> Defer<Self, I>;
}

impl<I, P: Handler<I, (), <P as Protocol>::Initial>> Session<P, I> {
    /// Create a new session initialized to the Protocol.
    pub fn new() -> Session<P, I> {
        let starting_func: DeferFunc<I, P, (), P::Initial> = Handler::<I, (), P::Initial>::with;

        Session(Defer(unsafe { mem::transmute(starting_func) }, PhantomData, true))
    }
}

impl<I, P: Handler<I, (), <<P as Protocol>::Initial as SessionType>::Dual>> Session<P, I> {
    /// Create a new session initialized to the dual of the Protocol.
    pub fn new_dual() -> Session<P, I> {
        let starting_func: DeferFunc<I, P, (), <P::Initial as SessionType>::Dual> = Handler::<I, (), <P::Initial as SessionType>::Dual>::with;

        Session(Defer(unsafe { mem::transmute(starting_func) }, PhantomData, true))
    }
}

impl<'a, P: Protocol, I> Session<P, I> {
    /// Operates the handler, returning false if the channel closed. This will panic
    /// if the channel was closed in a previous call.
    pub fn with(&mut self, io: &'a mut I) -> bool {
        // Construct a channel with a blank environment and session type.
        // These will be considered different, concrete types by the handler
        // we call.
        let p: Chan<'a, I, (), ()> = Chan(io, PhantomData);

        let new = ((self.0).0)(p);
        self.0 = new;

        return (self.0).2;
    }
}

impl<'a, I, E: SessionType, S: SessionType> Chan<'a, I, E, S> {
    /// Defer the rest of the protocol execution. Useful for returning early.
    /// 
    /// There must be a [`Handler`](trait.Handler.html) implemented for the protocol state you're deferring.
    pub fn defer<P: Handler<I, E, S>>(self) -> Defer<P, I> {
        let next_func: DeferFunc<I, P, E, S> = Handler::<I, E, S>::with;

        Defer(unsafe { mem::transmute(next_func) }, PhantomData, true)
    }
}

// TODO: refactor IO, add supertrait for close()
impl<'a, I: IO<usize>, E: SessionType> Chan<'a, I, E, Eps> {
    /// Close the channel. Only possible if it's in `Eps` (epsilon) state.
    pub fn close<P: Protocol>(self) -> Defer<P, I> {
        self.0.close();

        let next_func: DeferFunc<I, P, E, Eps> = Dummy::<I, P, E, Eps>::with;

        Defer(unsafe { mem::transmute(next_func) }, PhantomData, false)
    }
}

impl<'a, I: IO<A>, A, E: SessionType, S: SessionType> Chan<'a, I, E, Send<A, S>> {
    /// Send an `A` to IO.
    pub fn send(self, a: A) -> Chan<'a, I, E, S> {
        self.0.send(a);

        unsafe { mem::transmute(self) }
    }
}

impl<'a, I: IO<A>, A, E: SessionType, S: SessionType> Chan<'a, I, E, Recv<A, S>> {
    /// Receive an `A` from IO.
    pub fn recv(self) -> Result<(A, Chan<'a, I, E, S>), Self> {
        match self.0.recv() {
            Some(res) => Ok((res, unsafe { mem::transmute(self) })),
            None => {
                Err(self)
            }
        }
    }
}

impl<'a, I, E: SessionType, S: SessionType> Chan<'a, I, E, Nest<S>> {
    /// Enter into a nested protocol.
    pub fn enter(self) -> Chan<'a, I, (S, E), S> {
        unsafe { mem::transmute(self) }
    }
}

impl<'a, I, N: Peano, E: SessionType + Pop<N>> Chan<'a, I, E, Escape<N>> {
    /// Escape from a nested protocol.
    pub fn pop(self) -> Chan<'a, I, E::Tail, E::Head> {
        Chan(self.0, PhantomData)
    }
}

impl<'a, I: IO<u8>, E: SessionType, P: SessionType> Chan<'a, I, E, P> {
    /// Select a protocol to advance to.
    pub fn choose<S: SessionType>(self) -> Chan<'a, I, E, S> where P: Chooser<S> {
        self.0.send(P::num());

        Chan(self.0, PhantomData)
    }
}

impl<'a, I: IO<u8>, E: SessionType, S: SessionType, Q: SessionType> Chan<'a, I, E, Accept<S, Q>> {
    /// Accept one of many protocols and advance to its handler.
    pub fn accept<P: Protocol + Handler<I, E, Accept<S, Q>> + Acceptor<I, E, Accept<S, Q>>>(self) -> Defer<P, I> {
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


struct Dummy<I, P, E, S>(PhantomData<(I, P, E, S)>);
impl<I, P: Protocol, E: SessionType, S: SessionType> Dummy<I, P, E, S> {
    fn with<'a>(_: Chan<'a, I, E, S>) -> Defer<P, I> {
        panic!("Channel was closed!");
    }
}