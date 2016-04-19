# rq [![Build Status](https://travis-ci.org/dflemstr/rq.svg?branch=master)](https://travis-ci.org/dflemstr/rq)

**tl;dr:**

    curl -sSLf sh.dflemstr.name/rq | sh

This is the home of the tool called `rq` (record query).  I created it
out of frustration while manipulating data records on hundreds of
different machines and never having the right tools available.

This is one of my hack projects for personal learning so the code
quality is intentionally pretty low.  Everything is currently a work
in progress so don't expect to be able to use `rq` for productivity
right now.

`rq` is similar to [`awk`][awk] or [`jq`][jq] but supports more record
formats and operations.

Currently, the following input and output formats are supported:

  - Google Protocol Buffers
  - JSON

## Installation

The tool is distributed as a native statically linked binary.  The
quickest way to install it is:

    curl -sSLf sh.dflemstr.name/rq | sh

Currently, the following architectures are supported:

  - x86_64-unknown-linux-musl - Linux Intel 64-bit with zero
    dependencies; interfaces with syscalls directly
  - x86_64-unknown-linux-gnu - Linux Intel 64-bit with only GNU Lib C
    as the dependency.
  - i686-unknown-linux-gnu - Linux Intel 32-bit with only GNU Lib C as
    the dependency.
  - i686-apple-darwin - Mac OS X Intel 32-bit.
  - x86_64-apple-darwin - Mac OS X Intel 64-bit.

## Development

`rq` is written in the [Rust programming language][rust].  Assuming
that you have nothing installed and are in the `rq` directory, the
easiest way to set things up is to use `rustup`:

    # Install rust tools
    curl https://sh.rustup.rs -sSf | sh

    # Currently, rq requires the nightly rust compiler
    rustup override add nightly

    # Run tests
    cargo test

    # Build a debug executable
    cargo build

    # Build a release executable
    cargo build --release

[awk]: https://en.wikipedia.org/wiki/AWK
[jq]: https://stedolan.github.io/jq/
[rust]: https://www.rust-lang.org/
