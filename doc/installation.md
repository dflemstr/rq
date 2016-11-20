# Installation

There are many different ways to install `rq`.

  * [Generic](#generic)
  * [Manual download](#manual-download)
  * [GitHub releases](#github-releases)
  * [Arch Linux](#arch-linux)
  * [Mac OS X](#mac-os-x)

## Generic

There is a generic best-effort installer available via the dreaded
`curl | sh` method.  This is the preferred method, because you don't
need to compile `rq` from scratch, and you always get the latest
version.

    curl -sSLf https://sh.dflemstr.name/rq | sh

## Manual download

If you don't trust the above script (all it does is to detect your
architecture and run `curl`), you can also manually download `rq` for
your architecture.


  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-gnu/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-gnu/badge-small.svg?v=3"
           alt="x86_64-unknown-linux-gnu">
      x86_64-unknown-linux-gnu
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-musl/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-musl/badge-small.svg?v=3"
           alt="x86_64-unknown-linux-musl">
      x86_64-unknown-linux-musl
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-gnu/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-gnu/badge-small.svg?v=3"
           alt="i686-unknown-linux-gnu">
      i686-unknown-linux-gnu
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-musl/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-musl/badge-small.svg?v=3"
           alt="i686-unknown-linux-musl">
      i686-unknown-linux-musl
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-apple-darwin/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-apple-darwin/badge-small.svg?v=3"
           alt="x86_64-apple-darwin">
      x86_64-apple-darwin
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-apple-darwin/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-apple-darwin/badge-small.svg?v=3"
           alt="i686-apple-darwin">
      i686-apple-darwin
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabi/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabi/badge-small.svg?v=3"
           alt="arm-unknown-linux-gnueabi">
      arm-unknown-linux-gnueabi
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabi/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabi/badge-small.svg?v=3"
           alt="arm-unknown-linux-musleabi">
      arm-unknown-linux-musleabi
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabihf/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabihf/badge-small.svg?v=3"
           alt="arm-unknown-linux-gnueabihf">
      arm-unknown-linux-gnueabihf
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabihf/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabihf/badge-small.svg?v=3"
           alt="arm-unknown-linux-musleabihf">
      arm-unknown-linux-musleabihf
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-gnueabihf/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-gnueabihf/badge-small.svg?v=3"
           alt="armv7-unknown-linux-gnueabihf">
      armv7-unknown-linux-gnueabihf
    </a>
  * <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-musleabihf/rq">
      <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-musleabihf/badge-small.svg?v=3"
           alt="armv7-unknown-linux-musleabihf">
      armv7-unknown-linux-musleabihf
    </a>

You need to manually put the downloaded file in e.g. `/usr/local/bin`
and run `chmod +x` on it.

## GitHub releases

There are tagged releases of `rq` fairly infrequently.  You can
download pre-built binaries from the
[GitHub releases](https://github.com/dflemstr/rq/releases) page.  Note
that these might be very out of date compared to `master`.

## Arch Linux

There is a package available for AUR, so it can be installed with for
example `pacaur`:

    pacaur -S record-query-git

This takes a while to install because `rq` will be built from source.

## Mac OS X

There is a Homebrew tap available.  Add it like this:

    brew tap dflemstr/tools

This will let you install the latest version of `rq` (recommended):

    brew install --HEAD rq

Note that the compilation might take some time, use `-v` for details.

If you for some reason want the last tagged release of `rq` (might be
severely out of date):

    brew install rq
