#[macro_export]
macro_rules! proto(
/*    (@peano 0) => (Z);
    (@peano 1) => (S<Z>);
    (@peano 2) => (S<proto!(@peano 1)>);
    (@peano 3) => (S<proto!(@peano 2)>);
    (@peano 4) => (S<proto!(@peano 3)>);
    (@peano 5) => (S<proto!(@peano 4)>);
    (@peano 6) => (S<proto!(@peano 5)>);
    (@peano 7) => (S<proto!(@peano 6)>);
    (@peano 8) => (S<proto!(@peano 7)>);
    (@peano 9) => (S<proto!(@peano 8)>);
    (@peano 10) => (S<proto!(@peano 9)>);
    (@peano 11) => (S<proto!(@peano 10)>);
    (@peano 12) => (S<proto!(@peano 11)>);
    (@peano 13) => (S<proto!(@peano 12)>);
    (@peano 14) => (S<proto!(@peano 13)>);
    (@peano 15) => (S<proto!(@peano 14)>);
    (@peano 16) => (S<proto!(@peano 15)>);*/
    (@form_ty End) => (End);
//    (@form_ty loop { $($rest:tt)* }) => (Nest<proto!(@form_ty $($rest)*)>);
//    (@form_ty continue $p:tt) => (Escape<proto!(@peano $p)>);
//    (@form_ty continue) => (Escape<Z>);
    (@form_ty Goto $t:ty) => (Goto<$t>);
    (@form_ty Recv $t:ty, $($rest:tt)*) => (Recv<$t, proto!(@form_ty $($rest)*)>);
    (@form_ty Send $t:ty, $($rest:tt)*) => (Send<$t, proto!(@form_ty $($rest)*)>);
//    (@form_ty Choose {$p:tt, $($rest:tt)*}) => (Choose<proto!(@form_ty $p), proto!(@form_ty Choose {$($rest)*})>);
//    (@form_ty Choose {$p:tt}) => (Finally<proto!(@form_ty $p)>);
//    (@form_ty Accept {$p:tt, $($rest:tt)*}) => (Accept<proto!(@form_ty $p), proto!(@form_ty Accept {$($rest)*})>);
//    (@form_ty Accept {$p:tt}) => (Finally<proto!(@form_ty $p)>);
//    (@form_ty Accept {$p:ty = {$($inner:tt)*}}) => (Finally<proto!(@form_ty $p = {$($inner)*})>);
//    (@form_ty Choose {$p:ty = {$($inner:tt)*}}) => (Finally<proto!(@form_ty $p = {$($inner)*})>);
//    (@form_ty Choose {$p:ty = {$($inner:tt)*}, $($rest:tt)*}) => (Choose<proto!(@form_ty $p = {$($inner)*}), proto!(@form_ty Choose {$($rest)*})>);
//    (@form_ty Accept {$p:ty = {$($inner:tt)*}, $($rest:tt)*}) => (Accept<proto!(@form_ty $p = {$($inner)*}), proto!(@form_ty Accept {$($rest)*})>);
    (@form_ty {$($stuff:tt)*}) => (proto!(@form_ty $($stuff)*));
    (@form_ty $i:ty = {$($stuff:tt)*}) => (<$i as Alias>::Id);
    (@form_ty $i:ty = $t:ident {$($stuff:tt)*}) => (<$i as Alias>::Id);
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
    (@new_aliases ($alias:ident = $t:ident {$($astuff:tt)*} $($lol:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($lol)*) ($alias = {$t {$($astuff)*}}) $($others)*);
    );
    (@new_aliases ($x:ident $($rest:tt)*) $($others:tt)*) => (
        proto!(@new_aliases ($($rest)*) $($others)*);
    );
    (@construct_final ($alias:ident, $($arest:tt)*)) => (
        #[allow(dead_code)]
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
    ($start:ident = {$($rest:tt)*}) => (
        proto!(@construct_alias ($start = {$($rest)*}) @eps);
    );
);