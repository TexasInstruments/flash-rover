
--retain=g_pfnVectors
--entry_point ResetISR
--diag_suppress=10063
--heap_size=0
--stack_size=256
//--library=rtsv7M3_T_le_eabi.lib

#define FLASH_BASE            0x00000000
#define FLASH_SIZE            0x00058000

#define GPRAM_BASE            0x11000000
#define GPRAM_SIZE            0x00002000

#define SRAM_BASE             0x20000000
#define SRAM_SIZE             0x00014000

__STACK_TOP = __stack + __STACK_SIZE;

MEMORY
{
	FLASH (RX) : origin = FLASH_BASE, length = SRAM_SIZE
	SRAM (RWX) : origin = SRAM_BASE,  length = SRAM_SIZE
}

SECTIONS
{
	.intvecs     : > FLASH
	.text        : > FLASH
	.const       : > FLASH
	.constdata   : > FLASH
	.rodata      : > FLASH
	.cinit       : > FLASH
	.pinit       : > FLASH
	.init_array  : > FLASH
	.emb_text    : > FLASH
	.ccfg        : > FLASH (HIGH)

	.vtable      : > SRAM
	.vtable_ram  : > SRAM
	vtable_ram   : > SRAM
	.data        : > SRAM
	.bss         : > SRAM
	.sysmem      : > SRAM
	.stack       : > SRAM (HIGH)
	.nonretenvar : > SRAM
}
