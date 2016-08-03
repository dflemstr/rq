# Development

`rq` is mostly written in the [Rust programming language][rust].
Assuming that you have nothing installed, the easiest way to set
things up is to use `rustup` (see [rustup.rs](https://www.rustup.rs/)
for more info):

    curl -sSLf https://sh.rustup.rs | sh

The Rust installer will give you further platform-specific
instructions (e.g. if you're missing other development tools).

To build `rq`, navigate to the source directory, and switch to the
nightly Rust toolchain:

    rustup override add nightly

Now you can run the tests for the project (including JSDoc tests):

    cargo test

A debug build of the executable can be created like so:

    cargo build

It will be available in `target/debug/rq`.

A release build can be created like so (might take a lot longer):

    cargo build --release

It will be available in `target/release/rq`.

To build a version that doesn't depend on `glibc` (on Linux), first
add a new compiler target for `musl`:

    rustup target add x86_64-unknown-linux-musl

Then, install the `musl` standard C library (the package is usually
called `musl` or `musl-tools`).  This lets you do:

    cargo build --release --target x86_64-unknown-linux-musl

The resulting executable is available in
`target/x86_64-unknown-linux-musl/release/rq`.

[rust]: https://www.rust-lang.org/
