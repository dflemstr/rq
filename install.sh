#!/bin/sh -eu
# Copyright 2016 David Flemström.
#
# Based on https://sh.rustup.rs, which is:
# Copyright 2016 The Rust Project Developers
# Licensed under the Apache License, Version 2.0
# (http://www.apache.org/licenses/LICENSE-2.0)

base='https://s3-eu-west-1.amazonaws.com/record-query/record-query'

msg() {
    echo "$@" >&2
}

err() {
    msg "$@"
    exit 1
}

cpu="$(uname -m)"
os="$(uname -s)"

# Darwin `uname -s` lies
if [ "$os" = Darwin ] && [ "$cpu" = i386 ]
then
    if sysctl hw.optional.x86_64 | grep -q ': 1'
    then
        cpu=x86_64
    fi
fi

case "$os" in
    Linux)
        os=unknown-linux-gnu ;;
    FreeBSD)
        os=unknown-freebsd ;;
    DragonFly)
        os=unknown-dragonfly ;;
    Darwin)
        os=apple-darwin ;;
    MINGW* | MSYS* | CYGWIN*)
        os=pc-windows-gnu ;;
    *)
        err "unrecognized OS type: $os" ;;
esac

case "$cpu" in
    i386 | i486 | i686 | i786 | x86)
        cpu=i686 ;;
    xscale | arm)
        cpu=arm ;;
    armv6l)
        cpu=arm
        os="${os}eabihf" ;;
    armv7l)
        cpu=armv7
        os="${os}eabihf" ;;
    aarch64)
        cpu=aarch64 ;;
    x86_64 | x86-64 | x64 | amd64)
        cpu=x86_64 ;;
    *)
        err "unrecognized CPU type: $cpu" ;;
esac

# Detect 64-bit linux with 32-bit user land
if [ "$os" = unknown-linux-gnu ] && [ "$cpu" = x86_64 ]
then
    bin_to_probe="/usr/bin/env"
    if [ -e "$bin_to_probe" ]
    then
        file -L "$bin_to_probe" | grep -q "x86[_-]64"
        if [ $? != 0 ]
        then
            cpu=i686
        fi
    fi
fi

# if [ "$os" = unknown-linux-gnu ] && [ "$cpu" = x86_64 ]
# then
#     os=unknown-linux-musl
# fi

arch="$cpu-$os"
url="$base/$arch/rq"
path="/usr/local/bin/rq"

msg "Detected your architecture to be $arch"
msg "Will now download rq into $path (using sudo)"
sudo curl "$url" -o "$path"
sudo chmod +x "$path"
msg "rq is now installed"
