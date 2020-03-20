#!/bin/bash
#
# Copyright (c) 2020 , Texas Instruments.
# Licensed under the BSD-3-Clause license
# (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
# notice may not be copied, modified, or distributed except according to those terms.

set -ex

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
OUTPUT_DIR=${ROOT_DIR}/output

CARGO=${CARGO:=cargo}

build() {
    echo "Building flash-rover"
    cd "${ROOT_DIR}"
    "${CARGO}" build --release
}

install() {
    echo "Installing flash-rover"

    local install_dir=${OUTPUT_DIR}/flash-rover
    local cargo_out_dir=${ROOT_DIR}/target/release

    rm -rf "${install_dir}" 2> /dev/null
    mkdir -p "${install_dir}"

    if test -f "${cargo_out_dir}/ti-xflash.exe"; then
        cp -t "${install_dir}" "${ROOT_DIR}/scripts/cli-entry/flash-rover.bat"
        cp -t "${install_dir}" "${cargo_out_dir}/ti-xflash.exe"
    else
        cp -t "${install_dir}" "${ROOT_DIR}/scripts/cli-entry/flash-rover"
        cp -t "${install_dir}" "${cargo_out_dir}/ti-xflash"
    fi
}

main() {
    build
    install
}

main
