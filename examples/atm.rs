#![feature(type_macros)]

#[macro_use]
extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

// This is a basic example of using nemo's session types to describe a protocol.

#[derive(Copy, Clone)]
struct Atm {
    balance: u64
}

type Id = String;
type AtmProtocol = proto!(
    Recv Id, // get the account id
    loop {
        goto AtmMenu
    }
);

type AtmMenu = proto!(
    Accept {
        {goto AtmDeposit}, // user wants to deposit
        End // user is done
    }
);

//        {goto AtmWithdraw}, // user wants to withdraw
//        {goto AtmGetBalance}, // user wants to get balance

type AtmDeposit = proto!(
    Recv u64, // get the amount they're depositing
    Send u64, // tell them their new balance
    continue 0
);

type AtmWithdraw = proto!(
    Recv u64,  // get the amount they're withdrawing
    Send bool, // tell them if withdrawal succeeded
    continue 0
);

type AtmGetBalance = proto!(
    Send u64,
    continue 0
);

fn main() {
    use std::thread;
    use nemo::channels::Blocking;

    let bank = Atm {
        balance: 0
    };

    impl Protocol for Atm {
        type Initial = AtmProtocol;
    }

    let (server, client) = Blocking::new(bank, bank);

    impl<I: Transfers<String> + Transfers<u64> + Transfers<bool>, E: SessionType> Handler<I, (AtmMenu, E), End> for Atm {
        fn with(this: Channel<Self, I, (AtmMenu, E), End>) -> Defer<Self, I> {
            this.close()
        }
    }

    // TODO: this is dumb
    impl<I: Transfers<String> + Transfers<u64> + Transfers<bool>, E: SessionType> Handler<I, (AtmMenu, E), AtmMenu> for Atm {
        fn with(this: Channel<Self, I, (AtmMenu, E), AtmMenu>) -> Defer<Self, I> {
            this.accept()
        }
    }

    impl<I: Transfers<String> + Transfers<u64> + Transfers<bool>, E: SessionType> Handler<I, (AtmMenu, E), AtmDeposit> for Atm {
        fn with(this: Channel<Self, I, (AtmMenu, E), AtmDeposit>) -> Defer<Self, I> {
            match this.recv() {
                Ok((amt, mut this)) => {
                    this.proto.balance += amt;
                    let new_balance = this.proto.balance;
                    this.send(new_balance).pop().defer()
                },
                _ => panic!("Client unexpectedly dropped")
            }
        }
    }

    impl<I: Transfers<String> + Transfers<u64> + Transfers<bool>, E: SessionType> Handler<I, E, AtmProtocol> for Atm {
        fn with(this: Channel<Self, I, E, AtmProtocol>) -> Defer<Self, I> {
            match this.recv() {
                Ok((msg, this)) => {
                    this.enter().accept()
                },
                Err(_) => {
                    panic!("Client unexpectedly dropped");
                }
            }
        }
    }

    thread::spawn(move || {
        let mut server = server.defer();
        loop {
            if !server.with() {
                break;
            }
        }
    });

    thread::spawn(move || {
        match client.send("Sean".to_string())
                    .enter()
                    .choose::<<AtmDeposit as SessionType>::Dual>()
                    .send(100)
                    .recv() {
                        Ok((worked, client)) => {
                            client.pop().choose::<End>().close();
                            assert_eq!(worked, 100);
                            println!("Client finished. It was a success.");
                        },
                        Err(client) => {
                            panic!("Server unexpectedly dropped");
                        }
                    }
    });

    loop { }
}
