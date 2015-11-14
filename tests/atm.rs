#![feature(type_macros)]
#[macro_use]
extern crate nemo;

use nemo::*;
use nemo::session_types::*;
use nemo::io::nonblocking::NonBlocking;

#[test]
fn test() {
    proto!(Atm = {
        Recv usize,
        Goto Atm
    });

    let (server, client) = NonBlocking::new::<Atm>();

    let mut server = server.recv(|val, this| {
        this.goto(|this| {
            this.recv(|val, this| {
                panic!("received two things!");
            })
        })
    });

    server.with();

    let client = client.send(10).goto(|this| {
        this.send(15).goto(|this| {
            this.defer(|_| panic!())
        })
    });
    server.with();
}