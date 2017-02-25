# This script takes care of testing your crate

set -ex

main() {
    (cd serde-avro; cross build --target $TARGET)
    (cd serde-protobuf; cross build --target $TARGET)
    cross build --target $TARGET

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    (cd serde-avro; cross test --target $TARGET)
    (cd serde-protobuf; cross test --target $TARGET)
    cross test --target $TARGET

    cross run --target $TARGET --bin rq -- --help
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
