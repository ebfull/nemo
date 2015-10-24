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
        fn with(this: Channel<Self, I, E, Orig>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, Recv<usize, End>> for MyProtocol {
        fn with(this: Channel<Self, I, E, Recv<usize, End>>) -> Defer<Self, I> {
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
        fn with(this: Channel<Self, I, E, Other>) -> Defer<Self, I> {
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

    let (client1, client2) = Blocking::new::<MyProtocol>();

    let mut client1 = client1.defer();
    let mut client2 = client2.defer();

    assert_eq!(true, client1.with()); // send 10
    assert_eq!(false, client2.with()); // recv 10, send 15, close
    assert_eq!(false, client1.with()); // recv 15, panic "Worked!"
}