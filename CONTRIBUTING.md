# Contributing

Contributions to `rq` are very welcome; please track contributions via
the [issue tracker](https://github.com/dflemstr/rq/issues).

All issues are marked as either `Bug`s or `Issue`s.  They can also be
tagged with an experience level `E-` which is one of `E-easy`,
`E-medium`, `E-hard`, `E-mentor` and the most likely languages
involved in the change `L-` which can be `L-rust`, `L-js` or `L-c`.

`rq` is not directly affiliated with Spotify but the project still
adheres to its
[code of conduct](https://github.com/spotify/code-of-conduct/blob/master/code-of-conduct.md).

# Development

`rq` is mostly written in the [Rust programming language][rust].
Assuming that you have nothing installed, the easiest way to set
things up is to use `rustup` (see [rustup.rs](https://www.rustup.rs/)
for more info):

    curl -sSLf https://sh.rustup.rs | sh

The Rust installer will give you further platform-specific
instructions (e.g. if you're missing other development tools).

To build `rq`, navigate to the source directory.  Now you can run the
tests for the project (including JSDoc tests):

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
