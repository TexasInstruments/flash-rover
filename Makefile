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
$(error "Unable to find CCS executable")
endif

CCS_WORKSPACE := fw/workspace
PROJECTSPECS := $(shell ls fw/gcc/cc13x0-cc26x0/*.projectspec) \
                $(shell ls fw/gcc/cc13x2-cc26x2/*.projectspec)

OUTPUT_DIR := output/flash-rover

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
	@echo "Build CLI project"
	@cd cli && cargo build --release

.PHONY: cli-clean
cli-clean:
	@echo "Clean CLI project"
	@cd cli && cargo clean

.PHONY: output
output:
	@echo "Build output folder"
	@[ -f cli/target/release/flash-rover.exe ] \
		&& cp -t ${OUTPUT_DIR} cli/target/release/flash-rover.exe \
		|| cp -t ${OUTPUT_DIR} cli/target/release/flash-rover
	@cp -r -t ${OUTPUT_DIR} cli/dss
	@mkdir -p ${OUTPUT_DIR}/dss/fw
	@cp -t ${OUTPUT_DIR}/dss/fw $(shell ls fw/workspace/*/Firmware/*.bin)
	@tar -cf ${OUTPUT_DIR}.tar ${OUTPUT_DIR}

.PHONY: output-clean
output-clean:
	@echo "Clean output folder"
	@rm -rf ${OUTPUT_DIR} 2> /dev/null
	@mkdir -p ${OUTPUT_DIR}
