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

    impl<I: IO<String> + IO<usize> + IO<isize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, Orig>) -> Defer<Self, I> {
            this.choose::<SendIsize>().send(10).close()
        }
    }

    impl<I: IO<String> + IO<usize> + IO<isize>, E: SessionType> Handler<I, E, DualOrig> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, DualOrig>) -> Defer<Self, I> {
            this.accept()
        }
    }

    impl<I: IO<String> + IO<usize> + IO<isize>, E: SessionType> Handler<I, E, DualSendString> for MyProtocol {
        fn with<'a>(_: Channel<'a, Self, I, E, DualSendString>) -> Defer<Self, I> {
            panic!("fail")
        }
    }

    impl<I: IO<String> + IO<usize> + IO<isize>, E: SessionType> Handler<I, E, DualSendUsize> for MyProtocol {
        fn with<'a>(_: Channel<'a, Self, I, E, DualSendUsize>) -> Defer<Self, I> {
            panic!("fail")
        }
    }

    impl<I: IO<String> + IO<usize> + IO<isize>, E: SessionType> Handler<I, E, DualSendIsize> for MyProtocol {
        fn with<'a>(this: Channel<'a, Self, I, E, DualSendIsize>) -> Defer<Self, I> {
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

    let (mut io1, mut io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(&mut io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(&mut io2).defer();
    assert_eq!(false, client1.with(&mut io1)); // client1 chooses a protocol, sends 10, closes channel
    assert_eq!(true, client2.with(&mut io2)); // client2 accepts the protocol, defers
    assert_eq!(false, client2.with(&mut io2)); // client2 receives the isize, asserts it's 10, closes channel
}