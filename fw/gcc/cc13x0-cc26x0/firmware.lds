/*
 * Copyright (c) 2019 , Texas Instruments.
 * Licensed under the BSD-3-Clause license
 * (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
 * notice may not be copied, modified, or distributed except according to those terms.
 */

STACKSIZE = 512;

ENTRY(ResetISR)

SRAM_START       = 0x20000000;
SRAM_LENGTH      = 0x00004000;

XFLASH_BUF_START = 0x20004000;
XFLASH_BUF_SIZE  = 0x00001000;

MEMORY
{
    SRAM 	  (RWX) : ORIGIN = SRAM_START, 		 LENGTH = SRAM_LENGTH
    XLASH_BUF (RWX) : ORIGIN = XFLASH_BUF_START, LENGTH = XFLASH_BUF_SIZE
}

REGION_ALIAS("REGION_TEXT",       SRAM);
REGION_ALIAS("REGION_BSS",        SRAM);
REGION_ALIAS("REGION_DATA",       SRAM);
REGION_ALIAS("REGION_STACK",      SRAM);
REGION_ALIAS("REGION_ARM_EXIDX",  SRAM);
REGION_ALIAS("REGION_ARM_EXTAB",  SRAM);
REGION_ALIAS("REGION_XFLASH_BUF", XLASH_BUF);

_resetVecs_base_address = SRAM_START;

SECTIONS {

    PROVIDE (_resetVecs_base_address = DEFINED(_resetVecs_base_address)
    								 ? _resetVecs_base_address
    								 : 0x00000000);

    .resetVecs (_resetVecs_base_address) : AT (_resetVecs_base_address) {
        KEEP (*(.resetVecs))
    } > REGION_TEXT

    .ramVecs (NOLOAD) : ALIGN(256) {
        KEEP (*(vtable_ram))
    } > REGION_DATA

    .text : {
        CREATE_OBJECT_SYMBOLS
        *(.text)
        *(.text.*)
        . = ALIGN(0x4);
        KEEP (*(.ctors))
        . = ALIGN(0x4);
        KEEP (*(.dtors))
        . = ALIGN(0x4);
        __init_array_start = .;
        KEEP (*(.init_array*))
        __init_array_end = .;
        *(.init)
        *(.fini*)
    } > REGION_TEXT AT> REGION_TEXT

    PROVIDE (__etext = .);
    PROVIDE (_etext = .);
    PROVIDE (etext = .);

    .rodata : {
        *(.rodata)
        *(.rodata.*)
    } > REGION_TEXT AT> REGION_TEXT

    .data : ALIGN(4) {
        __data_load__ = LOADADDR (.data);
        __data_start__ = .;
        *(.data)
        *(.data.*)
        . = ALIGN (4);
        __data_end__ = .;
    } > REGION_DATA AT> REGION_TEXT

    .ARM.exidx : {
        __exidx_start = .;
        *(.ARM.exidx* .gnu.linkonce.armexidx.*)
        __exidx_end = .;
    } > REGION_ARM_EXIDX AT> REGION_ARM_EXIDX

    .ARM.extab : {
        *(.ARM.extab* .gnu.linkonce.armextab.*)
    } > REGION_ARM_EXTAB AT> REGION_ARM_EXTAB

    PROVIDE (_conf_base_address = DEFINED(_conf_base_address)
    								? _conf_base_address
    								: 0x20003000);

	.conf (_conf_base_address) : AT(_conf_base_address) {
		KEEP (*(.conf))
	} > REGION_TEXT

    PROVIDE (_doorbell_base_address = DEFINED(_doorbell_base_address)
    								? _doorbell_base_address
    								: 0x20003100);

	.doorbell (_doorbell_base_address) : AT(_doorbell_base_address) {
		KEEP (*(.doorbell))
	} > REGION_DATA

	.xflashbuf (NOLOAD) : ALIGN(256) {
		KEEP (*(.xflashbuf))
	} > REGION_XFLASH_BUF

    .bss : {
        __bss_start__ = .;
        *(.shbss)
        *(.bss)
        *(.bss.*)
        *(COMMON)
        . = ALIGN (4);
        __bss_end__ = .;
    } > REGION_BSS AT> REGION_BSS

    .stack (NOLOAD) : ALIGN(0x8) {
        _stack = .;
        __stack = .;
        KEEP(*(.stack))
        . += STACKSIZE;
        _stack_end = .;
        __stack_end = .;
    } > REGION_STACK AT> REGION_STACK
}
