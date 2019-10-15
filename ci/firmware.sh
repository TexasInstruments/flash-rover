#!/bin/bash

set -ex

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
FW_DIR=fw
CCS_WORKSPACE=${FW_DIR}/workspace

CCS_ROOT=${CCS_ROOT:=/opt/ti/ccs}

if test -f "${CCS_ROOT}/eclipse/eclipsec.exe"; then
    # Windows
    CCS_EXE=${CCS_ROOT}/eclipse/eclipsec.exe
elif test -f "${CCS_ROOT}/eclipse/eclipse"; then
    # Linux
    CCS_EXE=${CCS_ROOT}/eclipse/eclipse
elif test -f "${CCS_ROOT}/eclipse/ccstudio"; then
    # macOS
    CCS_EXE=${CCS_ROOT}/eclipse/ccstudio
else
    >&2 echo "Unable to find CCS exetuable"
    exit 1
fi

PROJECTSPECS=$(ls "${FW_DIR}"/gcc/cc13x0-cc26x0/*.projectspec \
                  "${FW_DIR}"/gcc/cc13x2-cc26x2/*.projectspec | \
               sed 's/^/-ccs.location /')

ccs_import() {
    echo "Importing CCS projects"
    rm -rf "${CCS_WORKSPACE}" 2> /dev/null
    mkdir -p "${CCS_WORKSPACE}"
    "${CCS_EXE}" \
        -noSplash \
        -data "${CCS_WORKSPACE}" \
        -application com.ti.ccstudio.apps.projectImport \
        -ccs.overwrite \
        -ccs.autoImportReferencedProjects true \
        ${PROJECTSPECS}
}

ccs_build() {
    echo "Building CCS projects"
    "${CCS_EXE}" \
        -noSplash \
        -data "${CCS_WORKSPACE}" \
        -application com.ti.ccstudio.apps.projectBuild \
        -ccs.workspace \
        -ccs.configuration Firmware \
        -ccs.buildType full
}

main() {
    ccs_import
    ccs_build
}

main
