use std::mem;
use std::marker::PhantomData;
use super::SessionType;
use protocol::{Protocol, Handler, Defer, DeferFunc};

/// This trait effectively posits that a protocol which handles `T` must
/// additionally handle other types. If `T` is an `Accept<P, Q>` the
/// protocol must handle `P` *and* be an `Acceptor` of `Q`. If `T` is 
/// a `Finally<P>` it must handle `P`.
pub trait Acceptor<I, E, T>: Protocol + Sized {
	fn defer(usize) -> Defer<Self, I>;
}
impl<I, E: SessionType, H: Protocol + Handler<I, E, P> + Acceptor<I, E, Q>, P: SessionType, Q: SessionType> Acceptor<I, E, Accept<P, Q>> for H {
	fn defer(num: usize) -> Defer<H, I> {
		if num == 0 {
			let next_func: DeferFunc<I, Self, E, P> = <Self as Handler<I, E, P>>::with;

			Defer(unsafe { mem::transmute(next_func) }, PhantomData, true)
		} else {
			<Self as Acceptor<I, E, Q>>::defer(num - 1)
		}
	}
}
impl<I, E: SessionType, H: Protocol + Handler<I, E, P>,                     P: SessionType>                 Acceptor<I, E, Finally<P>>   for H {
	fn defer(_: usize) -> Defer<H, I> {
		// regardless of num we cannot proceed further than Finally
		let next_func: DeferFunc<I, Self, E, P> = <Self as Handler<I, E, P>>::with;

		Defer(unsafe { mem::transmute(next_func) }, PhantomData, true)
	}
}

/// Choose from `P` or something in `Q`.
pub struct Choose<P: SessionType, Q: SessionType>(PhantomData<(P, Q)>);

unsafe impl<P: SessionType, Q: SessionType> SessionType for Choose<P, Q> {
	type Dual = Accept<P::Dual, Q::Dual>;
}

trait NotSame { }
impl NotSame for .. { }
impl<A> !NotSame for (A, A) { }

pub trait Chooser<T> {
	fn num() -> usize;
}

impl<P: SessionType, Q: SessionType> Chooser<P> for Choose<P, Q> {
	fn num() -> usize { 0 }
}

impl<P: SessionType> Chooser<P> for Finally<P> {
	fn num() -> usize { 0 }
}

impl<P: SessionType, S: SessionType, Q: SessionType + Chooser<S>> Chooser<S> for Choose<P, Q>
	where (S, P): NotSame
{
	fn num() -> usize { Q::num().checked_add(1).unwrap() }
}

/// Accept either `P` or something in `Q`.
pub struct Accept<P: SessionType, Q: SessionType>(PhantomData<(P, Q)>);

unsafe impl<P: SessionType, Q: SessionType> SessionType for Accept<P, Q> {
	type Dual = Choose<P::Dual, Q::Dual>;
}

/// Finally choose `P`.
pub struct Finally<P: SessionType>(PhantomData<P>);

unsafe impl<P: SessionType> SessionType for Finally<P> {
	type Dual = Finally<P::Dual>;
}

#[test]
fn check_choose_works() {
	use super::{Recv, Eps};

	type GetUsize = Recv<usize, Eps>;
	type GetU8 = Recv<u8, Eps>;
	type GetString = Recv<String, Eps>;
	type Getisize = Recv<isize, Eps>;

	type Proto = Choose<Getisize, Choose<GetString, Choose<GetU8, Finally<GetUsize>>>>;

	assert_eq!(<Proto as Chooser<Getisize>>::num(), 0);
	assert_eq!(<Proto as Chooser<GetString>>::num(), 1);
	assert_eq!(<Proto as Chooser<GetU8>>::num(), 2);
	assert_eq!(<Proto as Chooser<GetUsize>>::num(), 3);
}