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

proto!(Atm, Start = {
    Recv String,
    AtmMenu = {Accept {
        AtmDeposit = {
            Recv u64,
            Send u64,
            Goto AtmMenu
        },
        AtmWithdraw = {
            Recv u64,
            Send bool,
            Goto AtmMenu
        },
        AtmGetBalance = {
            Send u64,
            Goto AtmMenu
        },
        AtmEnd = {End}
    }}
});

handlers!(
    Atm(String, usize, u64, bool);

    this(alias AtmEnd) => {
        this.close()
    }

    this(alias AtmGetBalance) => {
        let cur_balance = this.proto.balance;
        this.send(cur_balance).goto().accept().ok().unwrap()
    }

    this(alias AtmDeposit) => {
        match this.recv() {
            Ok((amt, mut this)) => {
                this.proto.balance += amt;
                let new_balance = this.proto.balance;
                this.send(new_balance).goto().accept().ok().unwrap()
            },
            _ => panic!("Client unexpectedly dropped")
        }
    }

    this(alias AtmWithdraw) => {
        match this.recv() {
            Ok((amt, mut this)) => {
                if this.proto.balance < amt {
                    this.send(false)
                } else {
                    this.proto.balance -= amt;
                    this.send(true)
                }.goto().accept().ok().unwrap()
            },
            _ => panic!("Client unexpectedly dropped")
        }
    }

    this(alias Start) => {
        match this.recv() {
            Ok((_, this)) => {
                this.accept().ok().unwrap()
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
                    .choose::<<<AtmDeposit as Alias>::Id as SessionType>::Dual>()
                    .send(100)
                    .recv() {
                        Ok((worked, client)) => {
                            assert_eq!(worked, 100);
                            client.goto().choose::<End>().close();
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
