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

pub mod peano;
pub mod session_types;
pub mod channels;
mod protocol;

pub use protocol::{Channel, Defer, Protocol, Handler, channel, channel_dual};

#[macro_export]
macro_rules! proto(
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
    (@form_ty End) => (End);
    (@form_ty loop { $($rest:tt)* }) => (Nest<proto!(@form_ty $($rest)*)>);
    (@form_ty continue $p:tt) => (Escape<proto!(@peano $p)>);
    (@form_ty continue) => (Escape<Z>);
    (@form_ty Goto $t:ty) => (Goto<$t>);
    (@form_ty Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!(@form_ty $($rest)*)>);
    (@form_ty Send $t:ty, $($rest:tt)*) => (Send<$t, proto!(@form_ty $($rest)*)>);
    (@form_ty Choose {$p:tt, $($rest:tt)*}) => (Choose<proto!(@form_ty $p), proto!(@form_ty Choose {$($rest)*})>);
    (@form_ty Choose {$p:tt}) => (Finally<proto!(@form_ty $p)>);
    (@form_ty Accept {$p:tt, $($rest:tt)*}) => (Accept<proto!(@form_ty $p), proto!(@form_ty Accept {$($rest)*})>);
    (@form_ty Accept {$p:tt}) => (Finally<proto!(@form_ty $p)>);
    (@form_ty {$($stuff:tt)*}) => (proto!(@form_ty $($stuff)*));
    (@form_ty $i:ty = {$($stuff:tt)*}) => (<$i as Alias>::Id);
    (@new_aliases () $($others:tt)*) => (
        proto!(@construct_alias $($others)*);
    );
    (@new_aliases ({$($some:tt)*}$($rest:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($some)* $($rest)*) $($others)*);
    );
    (@new_aliases (, $($rest:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($rest)*) $($others)*);
    );
    (@new_aliases ($alias:ident = {$($astuff:tt)*} $($lol:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($lol)*) ($alias = {$($astuff)*}) $($others)*);
    );
    (@new_aliases ($x:ident $($rest:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($rest)*) $($others)*);
    );
    (@construct_final ($alias:ident, $($arest:tt)*)) => (
        #[allow(dead_code)]
        struct $alias;

        impl Alias for $alias {
            type Id = proto!(@form_ty $($arest)*);
        }
    );
    (@construct_final ($alias:ident, $($arest:tt)*) $($rest:tt)*) => (
        proto!(@construct_final ($alias, $($arest)*));
        proto!(@construct_final $($rest)*);
    );
    (@construct_alias @eps $($rest:tt)*) => (
        proto!(@construct_final $($rest)*);
    );
    (@construct_alias ($alias:ident = {$($rest:tt)*}) $($others:tt)*) => (
        proto!(@new_aliases ($($rest)*) $($others)* ($alias, $($rest)*));
    );
    ($proto:ty, $start:ident = {$($rest:tt)*}) => (
        impl Protocol for $proto {
            type Initial = <$start as Alias>::Id;
        }

        proto!(@construct_alias ($start = {$($rest)*}) @eps);
    );
);

#[macro_export]
macro_rules! handlers {
    (@clean_type alias $lol:ty) => (<$lol as Alias>::Id);
    (@clean_type dual $($lol:tt)*) => (<handlers!(@clean_type $($lol)*) as SessionType>::Dual);
    (@clean_type $($lol:tt)*) => (proto!(@form_ty $($lol)*));
    (
        $protocol:ident ($($impl_bound:ty),*);
        $chan:ident($($environment:tt)*) => $b:block
    ) => (
        impl<I: IO, E> Handler<I, E, handlers!(@clean_type $($environment)*)> for $protocol
            where $(I: Transfers<$impl_bound>, )* E: SessionType
        {
            fn with($chan: Channel<Self, I, E, handlers!(@clean_type $($environment)*)>) -> Defer<Self, I> {
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

/// This trait is implemented by backing IO structures to offer an
/// interface for bi-directional channels. Discriminants are sent
/// and received by `Channel` to indicate protocol changes; they
/// tend to be smaller numbers, and so a variable length integer
/// could be sent over a network instead of the raw `usize`.
///
/// As with all implementations of `Transfer<T>` for this concrete
/// `IO`, if `IO` can guarantee that the backing channel is not
/// accessed outside of these two traits, `Channel` can guarantee
/// that these methods are only called when the data is expected
/// over the channel. Over a network this expectation may not
/// meet reality as there is no guarantee that the other side of
/// the channel is implemented correctly. In that case,
/// deserialization may be necessary.
pub unsafe trait IO {
	/// Closes the channel.
    unsafe fn close(&mut self);

    /// Send a discriminant over the channel. Over a network a
    /// variable length integer would be ideal.
    unsafe fn send_discriminant(&mut self, usize);

    /// Receives a discriminant from the channel. Over a network a
    /// variable length integer would be ideal.
    unsafe fn recv_discriminant(&mut self) -> Option<usize>;
}

/// An implementation of this trait provides sending and receiving
/// functionality to `Channel` for an arbitrary `T`. `Channel` will
/// only ever call these functions if it expects a `T`, so long as
/// outside of this trait and `IO` the backing channel cannot be
/// accessed.
///
/// See the explanation on `IO` for more details.
pub unsafe trait Transfers<T>: IO {
    /// Sends an object from the handler to the outside channel.
    unsafe fn send(&mut self, T);

    /// Attempts to retrieve an object from the outside channel. This *can* block
    /// but it also might not, depending on the impl.
    unsafe fn recv(&mut self) -> Option<T>;
}