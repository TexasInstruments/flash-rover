#!/bin/bash

set -ex

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
OUTPUT_DIR=${ROOT_DIR}/output

VERSION=$1

CROSS=${CROSS:=cross}
DOCKER_COMPOSE=${DOCKER_COMPOSE:=docker-compose}

http_proxy=${http_proxy:=}
https_proxy=${https_proxy:=}

mk_tarball() {
    local target=$1
    local name=flash-rover-${VERSION}-${target}
    local tarball=${name}.tar.gz
    local target_dir=${OUTPUT_DIR}/${name}
    local stage_dir=${target_dir}/flash-rover
    local cargo_out_dir=${ROOT_DIR}/target/${target}/release

    rm -f "${tarball}" 2> /dev/null
    rm -rf "${target_dir}" 2> /dev/null
    mkdir -p "${stage_dir}"

    if test -f "${cargo_out_dir}/flash-rover.exe"; then
        cp -t "${stage_dir}" "${cargo_out_dir}/flash-rover.exe"
    else
        cp -t "${stage_dir}" "${cargo_out_dir}/flash-rover"
    fi

    (cd "${target_dir}" && \
        tar -czf "${tarball}" flash-rover && \
        mv "${tarball}" "${OUTPUT_DIR}/${tarball}")
}

cross_build() {
    local target=$1

    echo "Building ${target}"
    (cd "${ROOT_DIR}" && \
        "${CROSS}" build --release --target=${target})

    mk_tarball ${target}
}

docker_build() {
    local target=$1

    if [ "${target}" == "x86_64-apple-darwin" ]; then
        echo "Building ${target}"
        cd "${ROOT_DIR}"
        "${DOCKER_COMPOSE}" build --build-arg http_proxy=${http_proxy} --build-arg https_proxy=${https_proxy} x86_64-apple-darwin
        "${DOCKER_COMPOSE}" run --rm -e http_proxy=${http_proxy} -e https_proxy=${https_proxy} x86_64-apple-darwin sh -c 'PATH="$HOME/osxcross/target/bin:$PATH" CC=o64-clang cargo build --release --target=x86_64-apple-darwin'

        mk_tarball ${target}
    fi
}

main() {
    cross_build x86_64-unknown-linux-gnu
    cross_build x86_64-unknown-linux-musl
    cross_build x86_64-pc-windows-gnu
    docker_build x86_64-apple-darwin
}

main
