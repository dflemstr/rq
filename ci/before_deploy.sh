#!/bin/bash -ex
# This script takes care of building your crate and packaging it for release

main() {
    local deploy
    local revision

    deploy=target/deploy
    revision=$(git describe --tags)

    mkdir -p "$deploy/$TARGET"
    test -f Cargo.lock || cargo generate-lockfile

    export V8_LIBS=$PWD/v8-build/lib/libv8uber.a
    export V8_SOURCE=$PWD/v8-build

    cross build --bin rq --target "$TARGET" --release

    strip "target/$TARGET/release/rq"
    cp "target/$TARGET/release/rq" "$deploy/$TARGET/rq"

    cd $deploy
    tar czf "$CRATE_NAME-$revision-$TARGET.tar.gz" -- *

    mkdir -p "$deploy/$TARGET/$revision"
    curl "https://img.shields.io/badge/${TARGET//-/--}-${revision//-/--}-blue.png" > "$deploy/$TARGET/badge.png"
    curl "https://img.shields.io/badge/${TARGET//-/--}-${revision//-/--}-blue.svg" > "$deploy/$TARGET/badge.svg"
    curl "https://img.shields.io/badge/v-$(echo "$revision" | sed 's/-/--/g;s/v//')-blue.png" > "$deploy/$TARGET/badge-small.png"
    curl "https://img.shields.io/badge/v-$(echo "$revision" | sed 's/-/--/g;s/v//')-blue.svg" > "$deploy/$TARGET/badge-small.svg"

    cp "target/$TARGET/release/rq" "$deploy/$TARGET/$revision/rq"

    if [ ! -z "$GEN_JSDOC" ]
    then
        curl -sL https://deb.nodesource.com/setup_6.x | sudo -E bash -
        apt-get update && apt-get install nodejs
        sudo npm install -g jsdoc
        (cd js-doc; npm install)
        jsdoc -c js-doc/conf.json -d target/doc/js src/api.js src/prelude.js

        cd target/doc
        git init
        git config user.email 'nobody@dflemstr.name'
        git config user.name 'Travis CI'
        git add .
        git commit -m "Generate rustdoc for $TRAVIS_COMMIT"
        git push --force "https://${GITHUB_TOKEN}@github.com/dflemstr/rq.git" master:gh-pages
    fi
}

main
