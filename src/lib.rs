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
macro_rules! proto {
	(@peano 0) => (Z);
	(@peano 1) => (S<Z>);
	(@peano 2) => (S<proto!(@peano 1)>);
	(@peano 3) => (S<proto!(@peano 2)>);
	(@peano 4) => (S<proto!(@peano 3)>);
	(@peano 5) => (S<proto!(@peano 4)>);
	(@peano 6) => (S<proto!(@peano 5)>);
	(@peano 7) => (S<proto!(@peano 6)>);
	(@peano 8) => (S<proto!(@peano 7)>);
	(@peano 9) => (S<proto!(@peano 8)>);
	(@peano 10) => (S<proto!(@peano 9)>);
	(@peano 11) => (S<proto!(@peano 10)>);
	(@peano 12) => (S<proto!(@peano 11)>);
	(@peano 13) => (S<proto!(@peano 12)>);
	(@peano 14) => (S<proto!(@peano 13)>);
	(@peano 15) => (S<proto!(@peano 14)>);
	(@peano 16) => (S<proto!(@peano 15)>);
	(Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!($($rest)*)>);
	(Send $t:ty, $($rest:tt)*) => (Send<$t, proto!($($rest)*)>);
	(loop { $($rest:tt)* }) => (Nest<proto!($($rest)*)>);
	(continue $p:tt) => (Escape<proto!(@peano $p)>);
	(continue) => (Escape<Z>);
	(goto $p:ty) => ($p);
	(End) => (End);
	({$($rest:tt)*}) => (proto!($($rest)*));
	(Choose { $p:tt, $($rest:tt)*}) => (Choose<proto!($p), proto!(Choose {$($rest)*})>);
	(Choose { $p:tt }) => (Finally<proto!($p)>);
	(Accept { $p:tt, $($rest:tt)*}) => (Accept<proto!($p), proto!(Accept {$($rest)*})>);
	(Accept { $p:tt }) => (Finally<proto!($p)>);
}

#[macro_export]
macro_rules! handlers {
    (@final_entry $t:ty => $($rest:tt)*) => (handlers!(@final_entry $($rest)*));
    (@final_entry $t:ty) => ($t);
    (@nested_env $prev:ty, $cur:ty => $($rest:tt)*) => (handlers!(@nested_env ($cur, $prev), $($rest)*));
    (@nested_env $prev:ty, $cur:ty) => ($prev);
    (
        $protocol:ident ($($impl_bound:ty),*);
        $chan:ident($($environment:tt)*) => $b:block
    ) => (
        impl<I: IO, E> Handler<I, handlers!(@nested_env E, $($environment)*), handlers!(@final_entry $($environment)*)> for $protocol
            where $(I: Transfers<$impl_bound>, )* E: SessionType
        {
            fn with($chan: Channel<Self, I, handlers!(@nested_env E, $($environment)*), handlers!(@final_entry $($environment)*)>) -> Defer<Self, I> {
                $b
            }
        }
    );
    (
        $protocol:ident ($($impl_bound:ty),*);
        $chan:ident($($environment:tt)*) => $b:block

        $($rest:tt)*
    ) => (
        handlers!(
            $protocol($($impl_bound),*);
            $chan($($environment)*) => $b
        );

        handlers!(
            $protocol($($impl_bound),*);
            $($rest)*
        );
    );
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