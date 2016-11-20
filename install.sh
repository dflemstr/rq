#!/bin/bash -eu
# Copyright 2016 David Flemström.
#
# Based on https://sh.rustup.rs, which is:
# Copyright 2016 The Rust Project Developers
# Licensed under the Apache License, Version 2.0
# (http://www.apache.org/licenses/LICENSE-2.0)

base='https://s3-eu-west-1.amazonaws.com/record-query/record-query'

msg() {
    printf "\33[1mrq:\33[0m %s\n" "$*" >&2
}

err() {
    msg "$@"
    exit 1
}

interactive=true
path=$( (command -v rq | grep -Fv '/usr/bin/') || echo /usr/local/bin/rq)

while [[ $# -gt 1 ]]
do
    case "$1" in
        -y|--yes)
            interactive=false
            ;;
        -o|--output|-p|--path)
            path=$2
            shift
            ;;
        *)
            ;;
    esac
    shift
done

msg "Welcome to the rq installer!"
msg

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

arch="$cpu-$os"

msg "Detected your architecture to be $arch"

# musl mappings
case "$arch" in
    x86_64-unknown-linux-gnu)
        musl_arch=x86_64-unknown-linux-musl ;;
    i686-unknown-linux-gnu)
        musl_arch=i686-unknown-linux-musl ;;
    arm-unknown-linux-gnueabi)
        musl_arch=arm-unknown-linux-musleabi ;;
    arm-unknown-linux-gnueabihf)
        musl_arch=arm-unknown-linux-musleabihf ;;
    armv7-unknown-linux-gnueabihf)
        musl_arch=armv7-unknown-linux-musleabihf ;;
esac

if [ -n "$musl_arch" ]
then
    if [ "$interactive" = true ]
    then
        msg 'You can install the glibc or musl version of rq:'
        msg
        msg '  • The musl version is statically linked and with zero'
        msg '    dependencies (recommended).'
        msg '  • The glibc version is slightly smaller but depends on'
        msg '    recent versions of libstdc++ and glibc that you might'
        msg '    not have installed.'
        msg
        msg 'Which one do you prefer?'

        options=(musl glibc)
        PS3='Choice: '
        select opt in "${options[@]}"
        do
            case "$opt" in
                musl)
                    arch="$musl_arch"; break ;;
                glibc)
                    break ;;
                *)
                    msg "Invalid choice" ;;
            esac
        done < /dev/tty
    else
        msg 'Detected that your platform supports musl!'
        arch="$musl_arch"
    fi
    msg "Using architecture $arch"
fi

url="$base/$arch/rq"

if [ "$interactive" = true ]
then
    msg "Where should rq be installed? (default: $path)"
    read -rp 'Path: ' new_path < /dev/tty
    if [ -n "$new_path" ]
    then
        path=$(eval echo "$new_path")
    fi
fi

if [ -f "$path" ]
then
    if command -v md5 > /dev/null
    then md5tool=md5
    elif command -v md5sum > /dev/null
    then md5tool=md5sum
    fi

    if command -v python2 > /dev/null
    then pythontool=python2
    elif command -v python > /dev/null
    then pythontool=python
    fi

    if [ -n "$md5tool" ]
    then
        checksum=$("$pythontool" - "$path" <<EOF
import hashlib
import sys

chunk_size = 5 * 1024 * 1024
md5s = []

with open(sys.argv[1], 'rb') as f:
    while True:
        data = f.read(chunk_size)

        if not data:
            break

        md5s.append(hashlib.md5(data))

digests = b''.join(m.digest() for m in md5s)

md5 = hashlib.md5(digests)
print '%s-%s' % (md5.hexdigest(), len(md5s))
EOF
                )
    else
        checksum='0'
    fi
else
    checksum='0'
fi

msg "Downloading rq..."
tmppath=$(mktemp)
trap "rm -f $(printf '%q' "$tmppath")" EXIT
status=$(curl -Lf "$url" --write-out "%{http_code}" -H "If-None-Match: \"$checksum\"" --progress-bar -o "$tmppath")

if [ "$status" = 304 ]
then msg "You already have the latest version of rq in $path"
elif [[ "$status" == 2* ]]
then
    if [ -w "$(dirname "$path")" ]
    then
        msg "Installing rq into $path"
        mv "$tmppath" "$path"
        chmod +x "$path"
    else
        msg "Installing rq into $path (using sudo)"
        sudo /bin/sh -euc "
          mv $(printf '%q' "$tmppath") $(printf '%q' "$path")
          chmod 755 $(printf '%q' "$path")
          chown root:root $(printf '%q' "$path")
        "
    fi

    msg "rq is now installed"
else
    msg "Failed to download rq"
    msg "Status code: $status"
    msg "$(cat "$tmppath")"
fi
