// error-pattern:Worked!

extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

fn main() {
    use nemo::channels::Blocking;

    struct MyProtocol;

    type SendString = Send<String, End>;
    type SendUsize = Send<usize, End>;
    type SendIsize = Send<isize, End>;
    type Orig = Choose<SendString, Choose<SendUsize, Finally<SendIsize>>>;

    type DualSendString = Recv<String, End>;
    type DualSendUsize = Recv<usize, End>;
    type DualSendIsize = Recv<isize, End>;
    type DualOrig = Accept<DualSendString, Accept<DualSendUsize, Finally<DualSendIsize>>>;

    impl Protocol for MyProtocol {
        type Initial = Orig;
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with(this: Channel<Self, I, E, Orig>) -> Defer<Self, I> {
            this.choose::<SendIsize>().send(10).close()
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualOrig> for MyProtocol {
        fn with(this: Channel<Self, I, E, DualOrig>) -> Defer<Self, I> {
            this.accept()
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualSendString> for MyProtocol {
        fn with(_: Channel<Self, I, E, DualSendString>) -> Defer<Self, I> {
            panic!("fail")
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualSendUsize> for MyProtocol {
        fn with(_: Channel<Self, I, E, DualSendUsize>) -> Defer<Self, I> {
            panic!("fail")
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualSendIsize> for MyProtocol {
        fn with(this: Channel<Self, I, E, DualSendIsize>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, sess)) => {
                	if msg == 10 {
                		panic!("Worked!");
                	}

                    sess.close()
                },
                Err(_) => {
                    panic!("fail")
                }
            }
        }
    }

    let (io1, io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(io2).defer();
    assert_eq!(false, client1.with()); // client1 chooses a protocol, sends 10, closes channel
    assert_eq!(true, client2.with()); // client2 accepts the protocol, defers
    assert_eq!(false, client2.with()); // client2 receives the isize, asserts it's 10, closes channel
}