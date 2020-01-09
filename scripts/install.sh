#!/bin/bash

set -ex

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
CLI_DIR=${ROOT_DIR}/cli
FW_DIR=${ROOT_DIR}/fw
OUTPUT_DIR=${ROOT_DIR}/output

CARGO=${CARGO:=cargo}

build() {
    echo "Building flash-rover"
    cd "${CLI_DIR}"
    "${CARGO}" build --release
}

install() {
    echo "Installing flash-rover"

    local install_dir=${OUTPUT_DIR}/flash-rover
    local cargo_out_dir=${CLI_DIR}/target/release

    rm -rf "${install_dir}" 2> /dev/null
    mkdir -p "${install_dir}"

    if test -f "${cargo_out_dir}/flash-rover.exe"; then
        cp -t "${install_dir}" "${cargo_out_dir}/flash-rover.exe"
    else
        cp -t "${install_dir}" "${cargo_out_dir}/flash-rover"
    fi

    cp -r -t "${install_dir}" dss

    mkdir -p "${install_dir}/dss/fw"
    cp -t "${install_dir}/dss/fw" $(ls "${FW_DIR}"/workspace/*/Firmware/*.bin)
}

main() {
    build
    install
}

main
