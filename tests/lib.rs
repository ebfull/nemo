extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

#[test]
fn choosing_protocol() {
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
            panic!("should not have received a string..")
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualSendUsize> for MyProtocol {
        fn with(_: Channel<Self, I, E, DualSendUsize>) -> Defer<Self, I> {
            panic!("should not have received a usize..")
        }
    }

    impl<I: Transfers<String> + Transfers<usize> + Transfers<isize>, E: SessionType> Handler<I, E, DualSendIsize> for MyProtocol {
        fn with(this: Channel<Self, I, E, DualSendIsize>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, sess)) => {
                    assert_eq!(msg, 10);

                    sess.close()
                },
                Err(_) => {
                    panic!("expected to get a message...");
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

#[test]
fn recursive_protocol() {
    use nemo::channels::Blocking;

    struct MyProtocol;

    type Orig = Nest<OrigEntered>;
    type OrigEntered = Send<usize, AwaitingNumber>;
    type AwaitingNumber = Recv<usize, Escape<Z>>;

    type DualOrig = Nest<DualOrigEntered>;
    type DualOrigEntered = Recv<usize, Send<usize, Escape<Z>>>;

    impl Protocol for MyProtocol {
        type Initial = Orig;
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with(this: Channel<Self, I, E, Orig>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, (OrigEntered, E), OrigEntered> for MyProtocol {
        fn with(this: Channel<Self, I, (OrigEntered, E), OrigEntered>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, (OrigEntered, E), AwaitingNumber> for MyProtocol {
        fn with(this: Channel<Self, I, (OrigEntered, E), AwaitingNumber>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, this)) => {
                    assert_eq!(msg, 20);
                    this.pop().defer()
                },
                Err(_) => panic!("should have received a message")
            }
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, DualOrig> for MyProtocol {
        fn with(this: Channel<Self, I, E, DualOrig>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, (DualOrigEntered, E), DualOrigEntered> for MyProtocol {
        fn with(this: Channel<Self, I, (DualOrigEntered, E), DualOrigEntered>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, this)) => {
                    assert_eq!(msg, 10);
                    this.send(20).pop().defer()
                },
                Err(_) => panic!("should have received a message")
            }
        }
    }

    let (io1, io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(io2).defer();

    assert_eq!(true, client1.with()); // enters nesting
    assert_eq!(true, client1.with()); // sends 10 to client2
    assert_eq!(true, client2.with()); // enters nesting
    assert_eq!(true, client2.with()); // receives 10 from client1, sends back 20, pops out of nesting
    assert_eq!(true, client1.with()); // receives 20 from client2, pops out of nesting
    assert_eq!(true, client1.with()); // sends 10 to client2
    assert_eq!(true, client2.with()); // receives 10 from client1
}


#[test]
fn initialize_protocol() {
    use nemo::channels::Blocking;

    struct MyProtocol;

    type SendNumber = Send<usize, GetNumber>;
    type GetNumber = Recv<usize, End>;

    type GetNumberFirst = Recv<usize, SendNumberSecond>;
    type SendNumberSecond = Send<usize, End>;

    impl Protocol for MyProtocol {
        type Initial = SendNumber;
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, GetNumber> for MyProtocol {
        fn with(this: Channel<Self, I, E, GetNumber>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, session)) => {
                    assert_eq!(msg, 20);
                    session.defer()
                },
                Err(_) => {
                    panic!("should have received something")
                }
            }
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, GetNumberFirst> for MyProtocol {
        fn with(this: Channel<Self, I, E, GetNumberFirst>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, session)) => {
                    assert_eq!(msg, 10);
                    session.send(20).defer()
                },
                Err(_) => {
                    panic!("should have received something")
                }
            }
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, SendNumber> for MyProtocol {
        fn with(this: Channel<Self, I, E, SendNumber>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: Transfers<usize>, E: SessionType> Handler<I, E, End> for MyProtocol {
        fn with(this: Channel<Self, I, E, End>) -> Defer<Self, I> {
            this.close()
        }
    }

    let (io1, io2) = Blocking::new();

    let mut client1: Defer<MyProtocol, Blocking> = channel(io1).defer();
    let mut client2: Defer<MyProtocol, Blocking> = channel_dual(io2).defer();

    assert_eq!(true, client1.with()); // sends 10 to client2
    assert_eq!(true, client2.with()); // receives 10, sends 10 to client1
    assert_eq!(true, client1.with()); // receives 10 from client2
    assert_eq!(false, client2.with()); // End
    assert_eq!(false, client1.with()); // End
}