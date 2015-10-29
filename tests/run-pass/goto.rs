#![feature(type_macros)]
#![feature(trace_macros)]

//#[macro_use]
extern crate nemo;
use nemo::*;
use nemo::session_types::*;
use nemo::peano::*;
use nemo::channels::Blocking;

trace_macros!(true);
/*
	(Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!($($rest)*)>);
	(Send $t:ty, $($rest:tt)*) => (Send<$t, proto!($($rest)*)>);
	(loop { $($rest:tt)* }) => (Nest<proto!($($rest)*)>);
	(continue $p:tt) => (Escape<proto!(@peano $p)>);
	(continue) => (Escape<Z>);
	(goto $p:ty) => ($p);
	(End) => (End);
	({$($rest:tt)*}) => (proto!($($rest)*));
	(Choose { $p:tt, $($rest:tt)*}) => (Choose<proto!($p), proto!(Choose {$($rest)*})>);
	(Choose { $p:tt }) => (Finally<proto!($p)>);
	(Accept { $p:tt, $($rest:tt)*}) => (Accept<proto!($p), proto!(Accept {$($rest)*})>);
	(Accept { $p:tt }) => (Finally<proto!($p)>);
*/

macro_rules! proto(
	(@form_ty End) => (End);
	(@form_ty Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!(@form_ty $($rest)*)>);
	(@form_ty Send $t:ty, $($rest:tt)*) => (Send<$t, proto!(@form_ty $($rest)*)>);
	(@form_ty Choose {$p:tt, $($rest:tt)*}) => (Choose<proto!(@form_ty $p), proto!(@form_ty Choose {$($rest)*})>);
	(@form_ty Choose {$p:tt}) => (Finally<proto!(@form_ty $p)>);
	(@form_ty Accept {$p:tt, $($rest:tt)*}) => (Accept<proto!(@form_ty $p), proto!(@form_ty Accept {$($rest)*})>);
	(@form_ty Accept {$p:tt}) => (Finally<proto!(@form_ty $p)>);
	(@form_ty {$($stuff:tt)*}) => (proto!(@form_ty $($stuff)*));
	(@form_ty $i:ty = {$($stuff:tt)*}) => (<$i as Alias>::Id);
	(@new_aliases () $($others:tt)*) => (
		proto!(@construct_alias $($others)*);
	);
	(@new_aliases ({$($some:tt)*}$($rest:tt)*) $($others:tt)*) => (
		proto!(@new_aliases ($($some)* $($rest)*) $($others)*);
	);
	(@new_aliases (, $($rest:tt)*) $($others:tt)*) => (
		proto!(@new_aliases ($($rest)*) $($others)*);
	);
	(@new_aliases ($alias:ident = {$($astuff:tt)*} $($lol:tt)*) $($others:tt)*) => (
		proto!(@new_aliases ($($lol)*) ($alias = {$($astuff)*}) $($others)*);
	);
	(@new_aliases ($x:ident $($rest:tt)*) $($others:tt)*) => (
		proto!(@new_aliases ($($rest)*) $($others)*);
	);
	(@construct_final ($alias:ident, $($arest:tt)*)) => (
		struct $alias;

		impl Alias for $alias {
			type Id = proto!(@form_ty $($arest)*);
		}
	);
	(@construct_final ($alias:ident, $($arest:tt)*) $($rest:tt)*) => (
		proto!(@construct_final ($alias, $($arest)*));
		proto!(@construct_final $($rest)*);
	);
	(@construct_alias @eps $($rest:tt)*) => (
		proto!(@construct_final $($rest)*);
	);
	(@construct_alias ($alias:ident = {$($rest:tt)*}) $($others:tt)*) => (
		proto!(@new_aliases ($($rest)*) $($others)* ($alias, $($rest)*));
	);
	($proto:ty, $start:ident = {$($rest:tt)*}) => (
		impl Protocol for $proto {
			type Initial = <$start as Alias>::Id;
		}

		proto!(@construct_alias ($start = {$($rest)*}) @eps);
	);
);

proto!(Atm, Start = {
	Recv usize,
	Test = {
		Recv usize,
		Lol = {Choose {
			End,
			End
		}}
	}
});

trace_macros!(false);

fn main() {

}