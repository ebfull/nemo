// error-pattern:Worked!

extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

fn main() {
	use nemo::channels::Blocking;

    struct MyProtocol;

    type Orig = Send<usize, Recv<usize, End>>;
    type Other = Recv<usize, Send<usize, End>>;

    impl Protocol for MyProtocol {
        type Initial = Orig;
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Orig>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, Recv<usize, End>> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Recv<usize, End>>) -> Defer<Self, I> {
            match this.recv() {
            	Ok((msg, this)) => {
            		assert_eq!(msg, 15);

            		panic!("Worked!");

            		this.close();
            	},
            	Err(this) => {
            		panic!("fail")
            	}
            }
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, Other> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Other>) -> Defer<Self, I> {
            match this.recv() {
            	Ok((msg, this)) => {
            		assert_eq!(msg, 10);

            		this.send(15).close()
            	},
            	Err(this) => {
            		panic!("fail")
            	}
            }
        }
    }

    let (mut io1, mut io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(&mut io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(&mut io2).defer();

    assert_eq!(true, client1.with(&mut io1)); // send 10
    assert_eq!(false, client2.with(&mut io2)); // recv 10, send 15, close
    assert_eq!(false, client1.with(&mut io1)); // recv 15, panic "Worked!"
}