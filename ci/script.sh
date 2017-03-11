#!/bin/bash -ex
# This script takes care of testing your crate

main() {
    export V8_LIBS=$PWD/v8-build/lib/libv8uber.a
    export V8_SOURCE=$PWD/v8-build

    cross build --target "$TARGET"

    if [ ! -z "$DISABLE_TESTS" ]
    then return
    fi

    cross test --all --target "$TARGET"
}

# we don't run the "test phase" when doing tagged deploys
if [ -z "$TRAVIS_TAG" ]
then main
fi
