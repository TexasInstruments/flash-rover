CCS_ROOT ?= C:/ti/ccs
ECLIPSEC := ${CCS_ROOT}/eclipse/eclipsec

CCS_WORKSPACE := fw/workspace
PROJECTSPECS := $(shell ls fw/gcc/cc13x0-cc26x0/*.projectspec) \
                $(shell ls fw/gcc/cc13x2-cc26x2/*.projectspec)

OUTPUT_DIR := output/flash-rover

help:
	@echo "Makefile targets"
	@echo "  fw:      build all FW"
	@echo "  cli:     build CLI"
	@echo "  output:  package cli and fw into output/ folder"
	@echo "  all:     in order -> fw, cli, output"

all: fw cli output

clean: fw-clean cli-clean output-clean

.PHONY: fw
fw: fw-clean fw-build

.PHONY: fw-build
fw-build: fw-ccs-import fw-ccs-build

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
	${ECLIPSEC} \
		-noSplash \
		-data ${CCS_WORKSPACE} \
		-application com.ti.ccstudio.apps.projectImport \
		-ccs.overwrite \
		-ccs.autoImportReferencedProjects true \
		$(addprefix -ccs.location ,${PROJECTSPECS})

.PHONY: fw-ccs-build
fw-ccs-build:
	@echo "Build CCS projects"
	${ECLIPSEC} \
		-noSplash \
		-data ${CCS_WORKSPACE} \
		-application com.ti.ccstudio.apps.projectBuild \
		-ccs.workspace \
		-ccs.configuration Firmware \
		-ccs.buildType full

.PHONY: cli
cli: cli-clean cli-build

.PHONY: cli-build
cli-build:
	@echo "Build CLI project"
	@cd cli && cargo build --release

.PHONY: cli-clean
cli-clean:
	@echo "Clean CLI project"
	@cd cli && cargo clean

.PHONY: output
output: output-clean output-build

.PHONY: output-build
output-build:
	@echo "Build output folder"
	@[ -f cli/target/release/flash-rover.exe ] \
		&& cp -t ${OUTPUT_DIR} cli/target/release/flash-rover.exe \
		|| cp -t ${OUTPUT_DIR} cli/target/release/flash-rover
	@cp -r -t ${OUTPUT_DIR} cli/dss
	@cp -t ${OUTPUT_DIR}/dss/fw $(shell ls fw/workspace/*/Firmware/*.bin)

.PHONY: output-clean
output-clean:
	@echo "Clean output folder"
	@rm -rf ${OUTPUT_DIR} 2> /dev/null
	@mkdir -p ${OUTPUT_DIR}
