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

pub mod peano;
pub mod session_types;
pub mod channels;
mod protocol;

pub use protocol::{Chan, Defer, Session, Protocol, Handler};

/// This trait describes a backend for a channel to expose a message
/// passing interface.
pub trait IO<T> {
    /// Sends an object from the handler to the outside channel.
    fn send(&mut self, T);

    /// Attempts to retrieve an object from the outside channel. This *can* block
    /// but it also might not, depending on the impl.
    fn recv(&mut self) -> Option<T>;

    /// Closes the channel.
    fn close(&mut self);
}