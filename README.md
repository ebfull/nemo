# nemo [![Build status](https://api.travis-ci.org/ebfull/nemo.svg)](https://travis-ci.org/ebfull/nemo) [![Crates.io](https://img.shields.io/crates/v/nemo.svg)](https://crates.io/crates/nemo) #

### [Documentation](https://ebfull.github.io/nemo/)

Nemo is a Rust language **session types** library which focuses on asynchronous networking interfaces. Session types are a way of embedding a protocol into the typesystem, such that the implementation of it *must* be correct, or the code simply cannot compile. The standard of "correct" is that no client will disagree on the expectations or state of another. Session types can be used to enforce that only certain messages can be sent in particular orders over the network. Session types can embed much more complicated logic to handle protocols which involve nested or recursive state changes.

## How does nemo work?

The `session_types` crate is where nemo draws most of its inspiration. In order to support asynchronous channels and generic IO backends, it is designed differently so that you may *defer* a channel's handler to a future time -- perhaps when another event takes place on the network, or when it is convenient to resume work. If you never defer, there is no runtime cost, and when you do, the runtime cost is only *one* layer of indirection, sans code inlining. This change not only allows for async IO primitives, but also removes restrictions and requirements of end-user code.

Nemo provides an `IO` trait for implementing backends. As an example, nemo provides `nemo::channels::Blocking` which uses a backing bi-directional MPSC abstraction for safe communication between threads.

## Advantages to building network protocols with nemo
* Message tagging can be reduced or eliminated in some situations
* Complicated protocols can be described and implemented in a way that does not cause safety issues or race conditions/unexpected behavior
* IO backends can be swapped at any time, or wrapped for simulation purposes

----------------------------------

### Further work ahead
* Modifications may be needed to fully support things like `serde`/`bincode`/`nom`
* Macros or DSLs need to be implemented to simplify the protocol descriptions
* [#29205](https://github.com/rust-lang/rust/issues/29205)
* `Choose/Offer` style session types are not added yet; a clear approach which reduces network overhead must be considered.
* Heterogeneous selection is not yet available