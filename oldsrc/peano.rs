//! Peano numbers are natural numbers expressed as successions of zero.
//! We use them in our API to provide "nested" protocol escaping, as
//! you must indicate the number of layers you wish to escape from.

use std::marker::PhantomData;
use session_types::SessionType;

/// Represents a peano number.
pub unsafe trait Peano {}

/// Peano numbers: Zero
pub struct Z;
unsafe impl Peano for Z {}

/// Peano numbers: Increment
pub struct S<N> ( PhantomData<N> );
unsafe impl<N> Peano for S<N> {}

/// This represents the types obtained by popping N layers from
/// a stack.
pub trait Pop<N: Peano> {
    type Head: SessionType;
    type Tail: SessionType;
}

impl<A: SessionType, B: SessionType> Pop<Z> for (A, B) {
    type Head = A;
    type Tail = (A, B);
}

impl<N: Peano, A, B: Pop<N>> Pop<S<N>> for (A, B) {
    type Head = B::Head;
    type Tail = B::Tail;
}