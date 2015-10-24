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