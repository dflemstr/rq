# Installation

There are many different ways to install `rq`, listed from most preferred
to least preferred.

TODO: this section will soon be updated!

  * [Generic](#generic) (Up to date, fast)
  * [Cargo](#cargo) (Stable releases, slow)
  * [GitHub releases](#github-releases) (Stable releases, fast)
  * [Arch Linux](#arch-linux) (Up to date, slow)
  * [Mac OS X](#mac-os-x) (Out of date, slow)
  * [Nix](#nix) (Up to date, slow)

## Generic

There is a generic best-effort installer available via the dreaded
`curl | bash` method.  This is the preferred method, because you don't
need to compile `rq` from scratch, and you always get the latest
version.

    curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git dflemstr/rq

## Cargo

There is a crate available on [crates.io](https://crates.io/), so just run:

    cargo install record-query

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

## Nix

`rq` is available in nixpkgs. You can install it via `nix-env`:

    nix-env -i rq
    
 Or add to packages list if you use [Home Manager](https://github.com/rycee/home-manager):
 
     home.packages = [ pkgs.rq ]
