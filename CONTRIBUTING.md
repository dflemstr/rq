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

To build `rq`, navigate to the source directory.

You will need a build of the V8 Javascript engine as well.  If
your operating system package manager doesn't provide a package,
you can download a build like this:

    wget "https://s3-eu-west-1.amazonaws.com/record-query/v8/$TARGET/5.6.222/v8-build.tar.gz"
    tar -xvf v8-build.tar.gz
    export V8_LIBS=$PWD/v8-build/lib/libv8uber.a
    export V8_SOURCE=$PWD/v8-build

Now you can run the tests for the project (including JSDoc tests):

    cargo test

A debug build of the executable can be created like so:

    cargo build

It will be available in `target/debug/rq`.

A release build can be created like so (might take a lot longer):

    cargo build --release

It will be available in `target/release/rq`.

# Cross-compiled builds

The easiest way to create cross-compiled builds is to use the `./ci` script.

Look in the Travis build config for available parameters.  For example:

    TARGET=x86_64-unknown-linux-gnu USE_DOCKER=true ./ci test
    TARGET=x86_64-unknown-linux-gnu USE_DOCKER=true ./ci deploy

[rust]: https://www.rust-lang.org/
