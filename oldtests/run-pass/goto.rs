#![feature(type_macros)]
#![feature(trace_macros)]

#[macro_use]
extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;
use nemo::channels::Blocking;

struct Atm;

proto!(Atm, Start = {
	Recv usize,
	Test = {
		Choose {
			{rifk = {
				Recv usize,
				{Goto Test}
			}},
			{lofl = {
				Recv usize,
				End
			}}
		}
	}
});

handlers!(
	Atm(usize);

	this(alias Start) => {
		match this.recv() {
			Ok((num, this)) => {
				this.choose::<<rifk as Alias>::Id>().defer()
			},
			Err(this) => this.defer()
		}
	}

	this(alias rifk) => {
		this.defer()
	}
);

fn main() {

}