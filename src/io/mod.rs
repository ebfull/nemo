pub mod blocking;
pub mod nonblocking;

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