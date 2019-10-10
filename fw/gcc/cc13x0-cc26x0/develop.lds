
STACKSIZE = 512;

ENTRY(ResetISR)

FLASH_START       = 0x00000000;
FLASH_LENGTH      = 0x0001ffa8;

FLASH_CCFG_START  = 0x0001ffa8;
FLASH_CCFG_LENGTH = 0x00000058;

SRAM_START        = 0x20000000;
SRAM_LENGTH       = 0x00005000;

MEMORY
{
    FLASH (RX)      : ORIGIN = FLASH_START,      LENGTH = FLASH_LENGTH
    FLASH_CCFG (RX) : ORIGIN = FLASH_CCFG_START, LENGTH = FLASH_CCFG_LENGTH
    SRAM (RWX)      : ORIGIN = SRAM_START,       LENGTH = SRAM_LENGTH
}

REGION_ALIAS("REGION_TEXT",      FLASH);
REGION_ALIAS("REGION_BSS",       SRAM);
REGION_ALIAS("REGION_DATA",      SRAM);
REGION_ALIAS("REGION_STACK",     SRAM);
REGION_ALIAS("REGION_ARM_EXIDX", FLASH);
REGION_ALIAS("REGION_ARM_EXTAB", FLASH);
REGION_ALIAS("REGION_XFLASH_BUF", SRAM);

SECTIONS {

    PROVIDE (_resetVecs_base_address =
        DEFINED(_resetVecs_base_address) ? _resetVecs_base_address : 0x0);

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
	} > REGION_DATA

    PROVIDE (_doorbell_base_address = DEFINED(_doorbell_base_address)
    								? _doorbell_base_address
    								: 0x20003100);

	.doorbell (_doorbell_base_address) : AT(_doorbell_base_address) {
		KEEP (*(.doorbell))
	} > REGION_DATA

	.xflashbuf (NOLOAD) : ALIGN(256) {
		KEEP (*(.xflashbuf))
	} > REGION_XFLASH_BUF

    .ccfg : {
        KEEP (*(.ccfg))
    } > FLASH_CCFG AT> FLASH_CCFG

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
