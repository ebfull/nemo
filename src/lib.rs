//! Nemo provides session type abstractions for asynchronous networking
//! protocols. It can be used to build high performance, memory safe
//! and type-safe protocol implementations.
//!
//! ## What are session types?
//!
//! Session types allow you to encode the description of a protocol in
//! the type system. The goal is to ensure that two clients can never
//! disagree about their state or expectations when communicating.
//! Through session types, it is possible to define protocols that
//! *must* be implemented properly -- your code simply will not compile
//! otherwise.

#![feature(optin_builtin_traits)]

extern crate rand;

pub mod peano;
pub mod session_types;
pub mod channels;
mod protocol;

#[macro_export]
macro_rules! peano {
	() => (Z);
	(0) => (Z);
	(1) => (S<peano!(0)>);
	(2) => (S<peano!(1)>);
	(3) => (S<peano!(2)>);
	(4) => (S<peano!(3)>);
	(5) => (S<peano!(4)>);
	(6) => (S<peano!(5)>);
	(7) => (S<peano!(6)>);
	(8) => (S<peano!(7)>);
	(9) => (S<peano!(8)>);
	(10) => (S<peano!(9)>);
	(11) => (S<peano!(10)>);
}

#[macro_export]
macro_rules! proto {
	(Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!($($rest)*)>);
	(Send $t:ty, $($rest:tt)*) => (Send<$t, proto!($($rest)*)>);
	(loop { $($rest:tt)* }) => (Nest<proto!($($rest)*)>);
	(continue $p:tt) => (Escape<peano!($p)>);
	(goto $p:ty) => ($p);
	(End) => (End);
	({$($rest:tt)*}) => (proto!($($rest)*));
	(Choose { $p:tt, $($rest:tt)*}) => (Choose<proto!($p), proto!(Choose {$($rest)*})>);
	(Choose { $p:tt }) => (Finally<proto!($p)>);
	(Accept { $p:tt, $($rest:tt)*}) => (Accept<proto!($p), proto!(Accept {$($rest)*})>);
	(Accept { $p:tt }) => (Finally<proto!($p)>);
}

pub use protocol::{Channel, Defer, Protocol, Handler, channel, channel_dual};

pub unsafe trait IO {
	/// Closes the channel.
    unsafe fn close(&mut self);

    /// Send a variable length integer over the channel.
    unsafe fn send_varint(&mut self, usize);

    /// Receive a variable length integer from the channel.
    unsafe fn recv_varint(&mut self) -> Option<usize>;
}

/// This trait describes a backend for a channel to expose a message
/// passing interface. The trait is unsafe because it will be asked to supply
/// arbitrary types, and so must preserve the invariant that it will never allow
/// the backing channel to be modified outside of this trait.
pub unsafe trait Transfers<T>: IO {
    /// Sends an object from the handler to the outside channel.
    unsafe fn send(&mut self, T);

    /// Attempts to retrieve an object from the outside channel. This *can* block
    /// but it also might not, depending on the impl.
    unsafe fn recv(&mut self) -> Option<T>;
}