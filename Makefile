# Copyright (c) 2019 , Texas Instruments.
# Licensed under the BSD-3-Clause license
# (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
# notice may not be copied, modified, or distributed except according to those terms.

CCS_ROOT ?= C:/ti/ccs

# Windows
ifneq ("$(wildcard ${CCS_ROOT}/eclipse/eclipsec.exe)","")
CCS_CLI := ${CCS_ROOT}/eclipse/eclipsec.exe
# Linux
else ifneq ("$(wildcard ${CCS_ROOT}/eclipse/eclipse)","")
CCS_CLI := ${CCS_ROOT}/eclipse/eclipse
# macOS
else ifneq ("$(wildcard ${CCS_ROOT}/eclipse/ccstudio)","")
CCS_CLI := ${CCS_ROOT}/eclipse/ccstudio
else
$(warning "Unable to find CCS executable")
endif

VERSION = 0.1.0

TARGETS =x86_64-unknown-linux-gnu x86_64-apple-darwin x86_64-pc-windows-gnu

CCS_WORKSPACE := fw/workspace
PROJECTSPECS := $(shell ls fw/gcc/cc13x0-cc26x0/*.projectspec) \
                $(shell ls fw/gcc/cc13x2-cc26x2/*.projectspec)

OUTPUT_DIR := output

.PHONY: help
help:
	@echo "Makefile targets"
	@echo "  fw:      build all FW"
	@echo "  cli:     build CLI"
	@echo "  output:  package cli and fw into output/ folder"
	@echo "  build:   build all in order -> fw, cli, output"
	@echo "  clean:   clean all"
	@echo "  all:     clean all, then build all"

.PHONY: all
all: clean build

.PHONY: build
build: fw cli output

.PHONY: clean
clean: fw-clean cli-clean output-clean

.PHONY: fw
fw: fw-ccs-import fw-ccs-build

.PHONY: fw-clean
fw-clean: fw-ccs-clean

.PHONY: fw-ccs-clean
fw-ccs-clean:
	@echo "Clean CCS workspace"
	@rm -rf ${CCS_WORKSPACE} 2> /dev/null
	@mkdir -p ${CCS_WORKSPACE}

.PHONY: fw-ccs-import
fw-ccs-import:
	@echo "Import CCS projects"
	${CCS_CLI} \
		-noSplash \
		-data ${CCS_WORKSPACE} \
		-application com.ti.ccstudio.apps.projectImport \
		-ccs.overwrite \
		-ccs.autoImportReferencedProjects true \
		$(addprefix -ccs.location ,${PROJECTSPECS})

.PHONY: fw-ccs-build
fw-ccs-build:
	@echo "Build CCS projects"
	${CCS_CLI} \
		-noSplash \
		-data ${CCS_WORKSPACE} \
		-application com.ti.ccstudio.apps.projectBuild \
		-ccs.workspace \
		-ccs.configuration Firmware \
		-ccs.buildType full

.PHONY: cli
cli:
	@echo "Build CLI projects"
	@cd cli && ./build.sh

.PHONY: cli-clean
cli-clean:
	@echo "Clean CLI project"
	@cd cli && cargo clean

.PHONY: output
output:
	@echo "Build output folder"

	@for TARGET in ${TARGETS}; do \
		mkdir -p ${OUTPUT_DIR}/$${TARGET}/flash-rover; \
		[ -f cli/target/$${TARGET}/release/flash-rover.exe ] \
			&& cp -t ${OUTPUT_DIR}/$${TARGET}/flash-rover cli/target/$${TARGET}/release/flash-rover.exe \
			|| cp -t ${OUTPUT_DIR}/$${TARGET}/flash-rover cli/target/$${TARGET}/release/flash-rover; \
		cp -r -t ${OUTPUT_DIR}/$${TARGET}/flash-rover cli/dss; \
		mkdir -p ${OUTPUT_DIR}/$${TARGET}/flash-rover/dss/fw; \
		cp -t ${OUTPUT_DIR}/$${TARGET}/flash-rover/dss/fw $(shell ls fw/workspace/*/Firmware/*.bin); \
		cd ${OUTPUT_DIR}/$${TARGET}; \
		tar -czf flash-rover-${VERSION}-$${TARGET}.tar.gz flash-rover; \
		mv flash-rover-${VERSION}-$${TARGET}.tar.gz ..; \
		cd ../..; \
	done

.PHONY: output-clean
output-clean:
	@echo "Clean output folder"
	@rm -rf ${OUTPUT_DIR} 2> /dev/null
	@mkdir -p ${OUTPUT_DIR}
