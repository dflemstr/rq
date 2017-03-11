set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    source ~/.cargo/env || true

    cargo install --git https://github.com/dflemstr/cross.git --branch env

    if [ ! -d v8-build ]
    then
        wget "https://s3-eu-west-1.amazonaws.com/record-query/v8/$TARGET/5.7.441.1/v8-build.tar.gz"
        tar -xvf v8-build.tar.gz
    fi

    git describe --tags --always > git-version
}

main
