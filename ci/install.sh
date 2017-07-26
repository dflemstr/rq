#!/bin/bash -ex

main() {
    local cross_target=
    if [ "$TRAVIS_OS_NAME" = linux ]; then
        cross_target=x86_64-unknown-linux-musl
        sort=sort
    else
        cross_target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # This fetches latest stable release
    local tag
    tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
              | cut -d/ -f3 \
              | grep -E '^v[0.1.0-9.]+$' \
              | $sort --version-sort \
              | tail -n1)

    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag "$tag" \
           --target "$cross_target"

    if [ ! -d v8-build ]
    then
        wget "https://s3-eu-west-1.amazonaws.com/record-query/v8/$TARGET/5.7.441.1/v8-build.tar.gz"
        tar -xvf v8-build.tar.gz
    fi

    git describe --tags --always > git-version
}

main
