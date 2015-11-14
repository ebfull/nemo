use std::marker::PhantomData;

pub trait SessionType {
    type Dual: SessionType;
}

pub trait Alias {
	type Id: SessionType;
}

pub struct Send<T, P: SessionType>(PhantomData<(T, P)>);
pub struct Recv<T, P: SessionType>(PhantomData<(T, P)>);
pub struct End;
pub struct Goto<A: Alias>(PhantomData<A>);
pub struct GotoDual<A: Alias>(PhantomData<A>);

impl<A: Alias> SessionType for Goto<A> {
	type Dual = GotoDual<A>;
}

impl<A: Alias> SessionType for GotoDual<A> {
	type Dual = Goto<A>;
}

impl<T, P: SessionType> SessionType for Send<T, P> {
	type Dual = Recv<T, <P as SessionType>::Dual>;
}

impl<T, P: SessionType> SessionType for Recv<T, P> {
	type Dual = Send<T, <P as SessionType>::Dual>;
}

impl SessionType for End {
	type Dual = End;
}

