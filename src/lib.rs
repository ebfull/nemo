extern crate stack_dst;

pub mod session_types;
pub mod io;
#[macro_export]
pub mod macros;

use std::marker::PhantomData;
use io::*;
use session_types::*;
use macros::*;
use stack_dst::StackDST;

pub struct Channel<I: IO, S: SessionType> {
    io: I,
    _session: PhantomData<S>
}

impl<I: IO, S: SessionType> Channel<I, S> {
    fn new(io: I) -> Channel<I, S> {
        Channel {
            io: io,
            _session: PhantomData
        }
    }
}

impl<T, I: Transfers<T>, S: SessionType> Channel<I, Send<T, S>> {
    pub fn send(mut self, val: T) -> Channel<I, S> {
        unsafe { self.io.send(val); }

        Channel::new(self.io)
    }
}

pub struct Defer<I> {
    io: I,
    callback: StackDST<Fn(I) -> Defer<I>>
}

impl<I> Defer<I> {
    fn new<F: Fn(I) -> Defer<I> + 'static>(io: I, callback: F) -> Defer<I> {
        Defer {
            io: io,
            callback: StackDST::new(callback).ok().unwrap()
        }
    }

    pub fn with(&mut self) {
        use std::{mem,ptr};

        unsafe {
            let mut tmp: Defer<I> = mem::uninitialized();
            mem::swap(self, &mut tmp);

            let mut tmp = (tmp.callback)(tmp.io);
            mem::swap(self, &mut tmp);

            mem::forget(tmp);
        }
    }
}

impl<I: IO> Channel<I, End> {
    pub fn close(mut self) -> Defer<I> {
        unsafe { self.io.close() }

        Defer::new(self.io, |io| {
            panic!("trying to call a closed channel")
        })
    }
}

impl<T, I: Transfers<T>, S: SessionType> Channel<I, Recv<T, S>> {
    pub fn recv<F: Fn(T, Channel<I, S>) -> Defer<I> + 'static>(mut self, f: F) -> Defer<I> {
        match unsafe { self.io.recv() } {
            Some(val) => {
                f(val, Channel::new(self.io))
            },
            None => {
                Defer::new(self.io, move |io| {
                    use std::ptr;

                    let chan: Channel<I, Recv<T, S>> = Channel::new(io);

                    chan.recv(unsafe { ptr::read(&f) })
                })
            }
        }
    }
}

impl<I: IO, A: Alias> Channel<I, Goto<A>> {
    pub fn goto<F: Fn(Channel<I, <A as Alias>::Id>) -> Defer<I> + 'static>(self, f: F) -> Defer<I> {
        let chan = Channel::new(self.io);

        // TODO: defer instead of calling immediately
        // to avoid accidental stack overflow
        f(chan)
    }
}

impl<I: IO, A: Alias> Channel<I, GotoDual<A>> {
    pub fn goto<F: Fn(Channel<I, <<A as Alias>::Id as SessionType>::Dual>) -> Defer<I> + 'static>(self, f: F) -> Defer<I> {
        let chan = Channel::new(self.io);

        // TODO: defer instead of calling immediately
        // to avoid accidental stack overflow
        f(chan)
    }
}

impl<I: IO, S: SessionType> Channel<I, S> {
    pub fn defer<F: Fn(Channel<I, S>) -> Defer<I> + 'static>(mut self, f: F) -> Defer<I> {
        Defer::new(self.io, move |io| {
            let chan = Channel::new(io);

            f(chan)
        })
    }
}