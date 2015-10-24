use std::mem;
use std::marker::PhantomData;
use super::SessionType;
use protocol::{Channel, Protocol, Handler, Defer, DeferFunc};

/// This trait effectively posits that a protocol which handles `T` must
/// additionally handle other types. If `T` is an `Accept<S, Q>` the
/// protocol must handle `S` *and* be an `Acceptor` of `Q`. If `T` is 
/// a `Finally<S>` it must handle `S`.
pub trait Acceptor<I, E: SessionType, T>: Protocol + Sized {
	fn defer<Y: SessionType>(chan: Channel<Self, I, E, Y>, usize) -> Defer<Self, I>;
}
impl<I, E: SessionType, H: Protocol + Handler<I, E, S> + Acceptor<I, E, Q>, S: SessionType, Q: SessionType> Acceptor<I, E, Accept<S, Q>> for H {
	#[inline(always)]
	fn defer<Y: SessionType>(chan: Channel<Self, I, E, Y>, num: usize) -> Defer<H, I> {
		if num == 0 {
			let next_func: DeferFunc<Self, I, E, S> = <Self as Handler<I, E, S>>::with;

			Defer::new(chan, unsafe { mem::transmute(next_func) }, true)
		} else {
			<Self as Acceptor<I, E, Q>>::defer(chan, num - 1)
		}
	}
}
impl<I, E: SessionType, H: Protocol + Handler<I, E, S>,                     S: SessionType>                 Acceptor<I, E, Finally<S>>   for H {
	#[inline(always)]
	fn defer<Y: SessionType>(chan: Channel<Self, I, E, Y>, _: usize) -> Defer<H, I> {
		// regardless of num we cannot proceed further than Finally
		let next_func: DeferFunc<Self, I, E, S> = <Self as Handler<I, E, S>>::with;

		Defer::new(chan, unsafe { mem::transmute(next_func) }, true)
	}
}

/// Choose from `S` or something in `Q`.
pub struct Choose<S: SessionType, Q: SessionType>(PhantomData<(S, Q)>);

unsafe impl<S: SessionType, Q: SessionType> SessionType for Choose<S, Q> {
	type Dual = Accept<S::Dual, Q::Dual>;
}

trait NotSame { }
impl NotSame for .. { }
impl<A> !NotSame for (A, A) { }

/// This trait selects for the de-Bruijn index of a protocol embedded within
/// a `Choose` decision tree.
pub trait Chooser<T> {
	fn num() -> usize;
}

impl<S: SessionType, Q: SessionType> Chooser<S> for Choose<S, Q> {
	#[inline(always)]
	fn num() -> usize { 0 }
}

impl<S: SessionType> Chooser<S> for Finally<S> {
	#[inline(always)]
	fn num() -> usize { 0 }
}

impl<R: SessionType, S: SessionType, Q: SessionType + Chooser<S>> Chooser<S> for Choose<R, Q>
	where (S, R): NotSame
{
	#[inline(always)]
	fn num() -> usize { Q::num().checked_add(1).unwrap() }
}

/// Accept either `S` or something in `Q`.
pub struct Accept<S: SessionType, Q: SessionType>(PhantomData<(S, Q)>);

unsafe impl<S: SessionType, Q: SessionType> SessionType for Accept<S, Q> {
	type Dual = Choose<S::Dual, Q::Dual>;
}

/// Finally choose `S`.
pub struct Finally<S: SessionType>(PhantomData<S>);

unsafe impl<S: SessionType> SessionType for Finally<S> {
	type Dual = Finally<S::Dual>;
}

#[test]
fn check_choose_works() {
	use super::{Recv, End};

	type GetUsize = Recv<usize, End>;
	type GetU8 = Recv<u8, End>;
	type GetString = Recv<String, End>;
	type Getisize = Recv<isize, End>;

	type Proto = Choose<Getisize, Choose<GetString, Choose<GetU8, Finally<GetUsize>>>>;

	assert_eq!(<Proto as Chooser<Getisize>>::num(), 0);
	assert_eq!(<Proto as Chooser<GetString>>::num(), 1);
	assert_eq!(<Proto as Chooser<GetU8>>::num(), 2);
	assert_eq!(<Proto as Chooser<GetUsize>>::num(), 3);
}