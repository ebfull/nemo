extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

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

    impl<I: IO<usize>, E: SessionType> Handler<I, E, Orig> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, Orig>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, (OrigEntered, E), OrigEntered> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, (OrigEntered, E), OrigEntered>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, (OrigEntered, E), AwaitingNumber> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, (OrigEntered, E), AwaitingNumber>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, this)) => {
                    assert_eq!(msg, 20);
                    this.pop().defer()
                },
                Err(_) => panic!("should have received a message")
            }
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, DualOrig> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, DualOrig>) -> Defer<Self, I> {
            this.enter().defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, (DualOrigEntered, E), DualOrigEntered> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, (DualOrigEntered, E), DualOrigEntered>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, this)) => {
                    assert_eq!(msg, 10);
                    this.send(20).pop().defer()
                },
                Err(_) => panic!("should have received a message")
            }
        }
    }

    let (mut io1, mut io2) = Blocking::new();

    let mut client1: Session<MyProtocol, Blocking> = Session::new();
    let mut client2: Session<MyProtocol, Blocking> = Session::new_dual();

    assert_eq!(true, client1.with(&mut io1)); // enters nesting
    assert_eq!(true, client1.with(&mut io1)); // sends 10 to client2
    assert_eq!(true, client2.with(&mut io2)); // enters nesting
    assert_eq!(true, client2.with(&mut io2)); // receives 10 from client1, sends back 20, pops out of nesting
    assert_eq!(true, client1.with(&mut io1)); // receives 20 from client2, pops out of nesting
    assert_eq!(true, client1.with(&mut io1)); // sends 10 to client2
    assert_eq!(true, client2.with(&mut io2)); // receives 10 from client1
}

#[test]
fn initialize_protocol() {
    use nemo::channels::Blocking;

    struct MyProtocol;

    type SendNumber = Send<usize, GetNumber>;
    type GetNumber = Recv<usize, Eps>;

    type GetNumberFirst = Recv<usize, SendNumberSecond>;
    type SendNumberSecond = Send<usize, Eps>;

    impl Protocol for MyProtocol {
        type Initial = SendNumber;
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, GetNumber> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, GetNumber>) -> Defer<Self, I> {
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

    impl<I: IO<usize>, E: SessionType> Handler<I, E, GetNumberFirst> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, GetNumberFirst>) -> Defer<Self, I> {
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

    impl<I: IO<usize>, E: SessionType> Handler<I, E, SendNumber> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, SendNumber>) -> Defer<Self, I> {
            this.send(10).defer()
        }
    }

    impl<I: IO<usize>, E: SessionType> Handler<I, E, Eps> for MyProtocol {
        fn with<'a>(this: Chan<'a, I, E, Eps>) -> Defer<Self, I> {
            this.close()
        }
    }

    let (mut io1, mut io2) = Blocking::new();

    let mut client1: Session<MyProtocol, Blocking> = Session::new();
    let mut client2: Session<MyProtocol, Blocking> = Session::new_dual();

    assert_eq!(true, client1.with(&mut io1)); // sends 10 to client2
    assert_eq!(true, client2.with(&mut io2)); // receives 10, sends 10 to client1
    assert_eq!(true, client1.with(&mut io1)); // receives 10 from client2
    assert_eq!(false, client2.with(&mut io2)); // Eps
    assert_eq!(false, client1.with(&mut io1)); // Eps
}