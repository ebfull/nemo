//! Session types encode the current state of a communication channel. It is
//! not possible to change to another state without following the protocol.
//!
//! As an example, if a client is in state `Recv<usize, End>`, it cannot
//! do anything except receive a `usize`. And when it is finished, it will
//! be in state `End`, which means it can do nothing except close the channel.

mod choose;

use std::marker::PhantomData;
use peano::*;
pub use self::choose::{Chooser,Accept,Choose,Finally,Acceptor};

/// All session types have duality. Two clients that communicate will
/// always have a session type that is the dual of their counterpart.
///
/// As an example, the dual of `Recv<T, S>` is `Send<T, S::Dual>`.
/// That is, one client expects to receive T and switch to session S,
/// while the other expects to send T and switch to the dual of S.
pub unsafe trait SessionType {
    type Dual: SessionType;
}

pub struct Goto<A: Alias>(PhantomData<A>);
pub struct GotoDual<A: Alias>(PhantomData<A>);
unsafe impl<A: Alias> SessionType for Goto<A> {
	type Dual = GotoDual<A>;
}
unsafe impl<A: Alias> SessionType for GotoDual<A> {
	type Dual = Goto<A>;
}
pub trait Alias {
	type Id: SessionType;
}

/// The session is at the end of communication.
/// The channel can now be gracefully closed.
pub struct End;

unsafe impl SessionType for End {
    type Dual = End;
}

/// The session expects to send `T` and proceed to session `S`.
pub struct Send<T, S: SessionType> ( PhantomData<(T, S)> );

unsafe impl<T, S: SessionType> SessionType for Send<T, S> {
    type Dual = Recv<T, S::Dual>;
}

/// The session expects to receive `T` and proceed to session `S`.
pub struct Recv<T, S: SessionType> ( PhantomData<(T, S)> );

unsafe impl<T, S: SessionType> SessionType for Recv<T, S> {
    type Dual = Send<T, S::Dual>;
}

/// Protocols ocassionally do not follow a linear path of behavior. It may
/// be necessary to return to a previous "state" in the protocol. However,
/// this cannot be expressed in the typesystem, because the type will fold
/// over itself infinitely. Instead, `Nest<S>` and `Escape<N>` are provided.
/// These types allow you to "break" out of a nested scope in the protocol
/// by an arbitrary number of layers `N`.
pub struct Nest<S: SessionType> ( PhantomData<S> );

unsafe impl<S: SessionType> SessionType for Nest<S> {
    type Dual = Nest<S::Dual>;
}

/// Escape from a nested scope by an arbitrary number of layers `N`, using
/// peano numbers.
pub struct Escape<N: Peano> ( PhantomData<N> );

unsafe impl<N: Peano> SessionType for Escape<N> {
    type Dual = Escape<N>;
}

// TODO: understand the interactions and needs of these impls
unsafe impl SessionType for () {
    type Dual = ();
}

unsafe impl<S: SessionType, Q: SessionType> SessionType for (S, Q) {
    type Dual = (S, Q);
}
