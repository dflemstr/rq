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

    curl -sSLf sh.dflemstr.name/rq | sh

## Manual download

If you don't trust the above script (all it does is to detect your
architecture and run `curl`), you can also manually download `rq` for
your architecture.

  * [x86_64-unknown-linux-gnu](https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-gnu/rq)
  * [x86_64-unknown-linux-musl](https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-musl/rq)
  * [i686-unknown-linux-gnu](https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-gnu/rq)
  * [x86_64-apple-darwin](https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-apple-darwin/rq)
  * [i686-apple-darwin](https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-apple-darwin/rq)

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
