#![feature(type_macros)]

#[macro_use]
extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;

// This is a basic example of using nemo's session types to describe a protocol.

struct Atm {
	balance: u64
}

type Id = String;
type AtmProtocol = proto!(
	Recv Id, // get the account id
	loop {
		Accept {
			{goto AtmDeposit}, // user wants to deposit
			{goto AtmWithdraw}, // user wants to withdraw
			{goto AtmGetBalance}, // user wants to get balance
			End // user is done
		}
	}
);

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
/*
#[test]
fn test_atm() {
	use nemo::channels::Blocking;

	let (server, client) = Blocking::new::<Atm>();
}
*/