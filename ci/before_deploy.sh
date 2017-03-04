# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross build --bin rq --target $TARGET --release

    cp target/$TARGET/release/rq $stage/

    revision=$(git describe --tags)
    curl "https://img.shields.io/badge/${TARGET//-/--}-${revision//-/--}-blue.png" > "$stage/badge.png"
    curl "https://img.shields.io/badge/${TARGET//-/--}-${revision//-/--}-blue.svg" > "$stage/badge.svg"
    curl "https://img.shields.io/badge/v-$(echo $revision | sed 's/-/--/g;s/v//')-blue.png" > "$stage/badge-small.png"
    curl "https://img.shields.io/badge/v-$(echo $revision | sed 's/-/--/g;s/v//')-blue.svg" > "$stage/badge-small.svg"

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
