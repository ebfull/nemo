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

type AtmProtocol = proto!(
    Recv String, // get the account id
    loop {
        goto AtmMenu
    }
);

type AtmMenu = proto!(
    Accept {
        {goto AtmDeposit}, // user wants to deposit
        {goto AtmWithdraw}, // user wants to withdraw
        {goto AtmGetBalance}, // user wants to get balance
        End // user is done
    }
);

type AtmDeposit = proto!(
    Recv u64, // get the amount they're depositing
    Send u64, // tell them their new balance
    continue
);

type AtmWithdraw = proto!(
    Recv u64,  // get the amount they're withdrawing
    Send bool, // tell them if withdrawal succeeded
    continue
);

type AtmGetBalance = proto!(
    Send u64,
    continue
);

impl Protocol for Atm {
    type Initial = AtmProtocol;
}

handlers!(
    Atm(String, u64, bool);

    this(AtmMenu => End) => {
        this.close()
    }

    this(AtmMenu => AtmGetBalance) => {
        let cur_balance = this.proto.balance;
        this.send(cur_balance).pop().accept().ok().unwrap()
    }

    this(AtmMenu => AtmDeposit) => {
        match this.recv() {
            Ok((amt, mut this)) => {
                this.proto.balance += amt;
                let new_balance = this.proto.balance;
                this.send(new_balance).pop().accept().ok().unwrap()
            },
            _ => panic!("Client unexpectedly dropped")
        }
    }

    this(AtmMenu => AtmWithdraw) => {
        match this.recv() {
            Ok((amt, mut this)) => {
                if this.proto.balance < amt {
                    this.send(false)
                } else {
                    this.proto.balance -= amt;
                    this.send(true)
                }.pop().accept().ok().unwrap()
            },
            _ => panic!("Client unexpectedly dropped")
        }
    }

    this(AtmProtocol) => {
        match this.recv() {
            Ok((_, this)) => {
                this.enter().accept().ok().unwrap()
            },
            Err(_) => {
                panic!("Client unexpectedly dropped");
            }
        }
    }
);

fn main() {
    use std::thread;
    use nemo::channels::Blocking;

    let bank = Atm {
        balance: 0
    };

    let (server, client) = Blocking::new(bank, bank);

    thread::spawn(move || {
        match client.send("Sean".into())
                    .enter()
                    .choose::<<AtmDeposit as SessionType>::Dual>()
                    .send(100)
                    .recv() {
                        Ok((worked, client)) => {
                            assert_eq!(worked, 100);
                            client.pop().choose::<End>().close();
                        },
                        Err(_) => {
                            panic!("Server unexpectedly dropped");
                        }
                    }
    });

    let mut server = server.defer();
    loop {
        if !server.with() {
            break;
        }
    }
}
