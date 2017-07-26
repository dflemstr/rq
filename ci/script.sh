#!/bin/bash -ex
# This script takes care of testing your crate

main() {
    local basedir
    if [ "$TRAVIS_OS_NAME" = linux ]
    then basedir=/project
    else basedir=$PWD
    fi

    export V8_LIBS=$basedir/v8-build/lib/libv8uber.a
    export V8_SOURCE=$basedir/v8-build

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
