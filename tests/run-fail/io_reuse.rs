// error-pattern:assertion failed

extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

fn main() {
	use nemo::channels::Blocking;

    struct MyProtocol;

    type Orig = Nest<Send<usize, Escape<Z>>>;
    type Other = Nest<Recv<usize, Escape<Z>>>;

    impl Protocol for MyProtocol {
        type Initial = Orig;
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Orig>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, (Send<usize, Escape<Z>>, E), Send<usize, Escape<Z>>> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, (Send<usize, Escape<Z>>, E), Send<usize, Escape<Z>>>) -> Defer<Self, I> {
            this.send(10).pop().defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, Recv<usize, Escape<Z>>> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Recv<usize, Escape<Z>>>) -> Defer<Self, I> {
            this.defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, Other> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Other>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    let (mut io1, mut io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(&mut io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(&mut io2).defer();

    client1.with(&mut io1);
    client2.with(&mut io2);
    client1.with(&mut io2);
}