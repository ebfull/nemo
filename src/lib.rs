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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChannelClaim([u8; 20]);

impl ChannelClaim {
	pub fn new() -> ChannelClaim {
		use rand::{Rng,thread_rng};
		let mut rng = thread_rng();
		ChannelClaim(rng.gen())
	}
}

pub unsafe trait IO {
	/// `Channel` will claim this `IO` backend as its own using a special identifier.
	/// The other methods must ensure this identifier is equivalent, and panic otherwise.
	/// Attempts to claim twice should also panic: the IO backend is forever tainted.
	fn claim(&mut self, id: ChannelClaim);

    /// Closes the channel.
    fn close(&mut self, id: ChannelClaim);
}

/// This trait describes a backend for a channel to expose a message
/// passing interface. The trait is unsafe because it will be asked to supply
/// arbitrary types, and so must preserve the invariant that it will never allow
/// the backing channel to be modified outside of this trait.
pub unsafe trait Transfers<T>: IO {
    /// Sends an object from the handler to the outside channel.
    fn send(&mut self, T, id: ChannelClaim);

    /// Attempts to retrieve an object from the outside channel. This *can* block
    /// but it also might not, depending on the impl.
    fn recv(&mut self, id: ChannelClaim) -> Option<T>;
}