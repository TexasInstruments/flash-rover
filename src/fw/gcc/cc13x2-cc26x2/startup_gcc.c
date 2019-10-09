
//*****************************************************************************
//
// Check if compiler is GNU Compiler
//
//*****************************************************************************
#if !(defined(__GNUC__))
#error "startup_gcc.c: Unsupported compiler!"
#endif

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(inc/hw_types.h)
#include DeviceFamily_constructPath(driverlib/setup.h)

//*****************************************************************************
//
// Macro for weak symbol aliasing
//
//*****************************************************************************
#define WEAK_ALIAS(x) __attribute__ ((weak, alias(#x)))

//*****************************************************************************
//
// Forward declaration of the reset ISR and the default fault handlers.
//
//*****************************************************************************
void        ResetISR( void );
static void NmiSRHandler( void );
static void FaultISRHandler( void );
static void IntDefaultHandler( void );
extern int  main( void );


// Default interrupt handlers
void NmiSR(void) WEAK_ALIAS(NmiSRHandler);
void FaultISR(void) WEAK_ALIAS(FaultISRHandler);
void MPUFaultIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void BusFaultIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void UsageFaultIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void SVCallIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void DebugMonIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void PendSVIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void SysTickIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void GPIOIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void I2CIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void RFCCPE1IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void PKAIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AONRTCIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void UART0IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXSWEvent0IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void SSI0IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void SSI1IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void RFCCPE0IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void RFCHardwareIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void RFCCmdAckIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void I2SIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXSWEvent1IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void WatchdogIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer0AIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer0BIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer1AIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer1BIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer2AIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer2BIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer3AIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void Timer3BIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void CryptoIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void uDMAIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void uDMAErrIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void FlashIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void SWEvent0IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXCombEventIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AONProgIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void DynProgIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXCompAIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXADCIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void TRNGIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void OSCIntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void AUXTimer2IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void UART1IntHandler(void) WEAK_ALIAS(IntDefaultHandler);
void BatMonIntHandler(void) WEAK_ALIAS(IntDefaultHandler);


//*****************************************************************************
//
// The following are constructs created by the linker, indicating where the
// the "data" and "bss" segments reside in memory.
//
//*****************************************************************************
extern uint32_t __data_load__;
extern uint32_t __data_start__;
extern uint32_t __data_end__;
extern uint32_t __bss_start__;
extern uint32_t __bss_end__;
extern uint32_t __stack_end;

//*****************************************************************************
//
//! The vector table. Note that the proper constructs must be placed on this to
//! ensure that it ends up at physical address 0x0000.0000 or at the start of
//! the program if located at a start address other than 0.
//
//*****************************************************************************
__attribute__ ((section(".resetVecs"), used))
void (* const g_pfnVectors[])(void) =
{
    (void (*)(void))((unsigned long)&__stack_end),
                                            //  0 The initial stack pointer
    ResetISR,                               //  1 The reset handler
    NmiSR,                                  //  2 The NMI handler
    FaultISR,                               //  3 The hard fault handler
    MPUFaultIntHandler,                     //  4 Memory Management (MemManage) Fault
    BusFaultIntHandler,                     //  5 The bus fault handler
    UsageFaultIntHandler,                   //  6 The usage fault handler
    0,                                      //  7 Reserved
    0,                                      //  8 Reserved
    0,                                      //  9 Reserved
    0,                                      // 10 Reserved
    SVCallIntHandler,                       // 11 Supervisor Call (SVCall)
    DebugMonIntHandler,                     // 12 Debug monitor handler
    0,                                      // 13 Reserved
    PendSVIntHandler,                       // 14 The PendSV handler
    SysTickIntHandler,                      // 15 The SysTick handler
    //--- External interrupts ---
    GPIOIntHandler,                         // 16 AON edge detect
    I2CIntHandler,                          // 17 I2C
    RFCCPE1IntHandler,                      // 18 RF Core Command & Packet Engine 1
    PKAIntHandler,                          // 19 PKA Interrupt event
    AONRTCIntHandler,                       // 20 AON RTC
    UART0IntHandler,                        // 21 UART0 Rx and Tx
    AUXSWEvent0IntHandler,                  // 22 AUX software event 0
    SSI0IntHandler,                         // 23 SSI0 Rx and Tx
    SSI1IntHandler,                         // 24 SSI1 Rx and Tx
    RFCCPE0IntHandler,                      // 25 RF Core Command & Packet Engine 0
    RFCHardwareIntHandler,                  // 26 RF Core Hardware
    RFCCmdAckIntHandler,                    // 27 RF Core Command Acknowledge
    I2SIntHandler,                          // 28 I2S
    AUXSWEvent1IntHandler,                  // 29 AUX software event 1
    WatchdogIntHandler,                     // 30 Watchdog timer
    Timer0AIntHandler,                      // 31 Timer 0 subtimer A
    Timer0BIntHandler,                      // 32 Timer 0 subtimer B
    Timer1AIntHandler,                      // 33 Timer 1 subtimer A
    Timer1BIntHandler,                      // 34 Timer 1 subtimer B
    Timer2AIntHandler,                      // 35 Timer 2 subtimer A
    Timer2BIntHandler,                      // 36 Timer 2 subtimer B
    Timer3AIntHandler,                      // 37 Timer 3 subtimer A
    Timer3BIntHandler,                      // 38 Timer 3 subtimer B
    CryptoIntHandler,                       // 39 Crypto Core Result available
    uDMAIntHandler,                         // 40 uDMA Software
    uDMAErrIntHandler,                      // 41 uDMA Error
    FlashIntHandler,                        // 42 Flash controller
    SWEvent0IntHandler,                     // 43 Software Event 0
    AUXCombEventIntHandler,                 // 44 AUX combined event
    AONProgIntHandler,                      // 45 AON programmable 0
    DynProgIntHandler,                      // 46 Dynamic Programmable interrupt
                                            //    source (Default: PRCM)
    AUXCompAIntHandler,                     // 47 AUX Comparator A
    AUXADCIntHandler,                       // 48 AUX ADC new sample or ADC DMA
                                            //    done, ADC underflow, ADC overflow
    TRNGIntHandler,                         // 49 TRNG event
    OSCIntHandler,                          // 50 Combined event from Oscillator control
    AUXTimer2IntHandler,                    // 51 AUX Timer2 event 0
    UART1IntHandler,                        // 52 UART1 combined interrupt
    BatMonIntHandler                        // 53 Combined event from battery monitor
};


//*****************************************************************************
//
//! This is the code that gets called when the processor first starts execution
//! following a reset event. Only the absolutely necessary set is performed,
//! after which the application supplied entry() routine is called. Any fancy
//! actions (such as making decisions based on the reset cause register, and
//! resetting the bits in that register) are left solely in the hands of the
//! application.
//
//*****************************************************************************
void
ResetISR(void)
{
    uint32_t *pSrc;
    uint32_t *pDest;

    //
    // Final trim of device
    //
    SetupTrimDevice();
    
    //
    // Copy the data segment initializers from FLASH to SRAM.
    //
    pSrc = &__data_load__;
    for(pDest = &__data_start__; pDest < &__data_end__; )
    {
        *pDest++ = *pSrc++;
    }

    //
    // Zero fill the bss segment.
    //
    __asm("    ldr     r0, =__bss_start__\n"
          "    ldr     r1, =__bss_end__\n"
          "    mov     r2, #0\n"
          "    .thumb_func\n"
          "zero_loop:\n"
          "        cmp     r0, r1\n"
          "        it      lt\n"
          "        strlt   r2, [r0], #4\n"
          "        blt     zero_loop");

    //
    // Enable the FPU
    // CPACR is located at address 0xE000ED88
    // Set bits 20-23 in CPACR to enable CP10 and CP11 coprocessors
    //
    __asm("    ldr.w   r0, =0xE000ED88\n"
          "    ldr     r1, [r0]\n"
          "    orr     r1, r1, #(0xF << 20)\n"
          "    str     r1, [r0]\n");

    //
    // Call the application's entry point.
    //
    main();

    //
    // If we ever return signal Error
    //
    FaultISR();
}

//*****************************************************************************
//
//! This is the code that gets called when the processor receives a NMI. This
//! simply enters an infinite loop, preserving the system state for examination
//! by a debugger.
//
//*****************************************************************************
static void
NmiSRHandler(void)
{
    //
    // Enter an infinite loop.
    //
    while(1)
    {
    }
}

//*****************************************************************************
//
//! This is the code that gets called when the processor receives a fault
//! interrupt. This simply enters an infinite loop, preserving the system state
//! for examination by a debugger.
//
//*****************************************************************************
static void
FaultISRHandler(void)
{
    //
    // Enter an infinite loop.
    //
    while(1)
    {
    }
}

//*****************************************************************************
//
//! This is the code that gets called when the processor receives an unexpected
//! interrupt. This simply enters an infinite loop, preserving the system state
//! for examination by a debugger.
//
//*****************************************************************************
static void
IntDefaultHandler(void)
{
    //
    // Go into an infinite loop.
    //
    while(1)
    {
    }
}
