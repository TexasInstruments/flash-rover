
#ifndef UART_HPP_
#define UART_HPP_

#include <stddef.h>
#include <stdint.h>

#include "power.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/ioc.h)
#include DeviceFamily_constructPath(driverlib/uart.h)
#include DeviceFamily_constructPath(inc/hw_memmap.h)

namespace bsp {

struct UartObj
{
    Power::Periph periph;
    uint32_t base;
    struct
    {
        uint32_t rx;
        uint32_t tx;
        uint32_t cts;
        uint32_t rts;
    } pins;
};

constexpr const UartObj uart0Obj = {
    Power::Periph::Uart0,  /* periph */
    UART0_BASE,            /* base */
    {                      /* pins */
         IOID_12,          /* rx */
         IOID_13,          /* tx */
         IOID_UNUSED,      /* cts */
         IOID_UNUSED,      /* rts */
    },
};

constexpr const UartObj uart1Obj = {
    Power::Periph::Uart1,  /* periph */
    UART1_BASE,            /* base */
    {                      /* pins */
         IOID_UNUSED,      /* rx */
         IOID_UNUSED,      /* tx */
         IOID_UNUSED,      /* cts */
         IOID_UNUSED,      /* rts */
    },
};

class Uart
{
private:
    UartObj             obj_;
    Power::PeriphHandle periph_;

public:
    Uart(const UartObj& obj, Power& power)
        : obj_{ obj }
        , periph_{ power.openPeriph(obj_.periph) }
    {
        UARTDisable(obj_.base);
        UARTConfigSetExpClk(obj_.base,
            48000000,
            115200,
            UART_CONFIG_WLEN_8 | UART_CONFIG_STOP_ONE | UART_CONFIG_PAR_NONE
        );
        UARTIntClear(obj_.base, UART_INT_OE | UART_INT_BE | UART_INT_PE |
                                UART_INT_FE | UART_INT_RT | UART_INT_TX |
                                UART_INT_RX | UART_INT_CTS);
        UARTFIFOLevelSet(obj_.base, UART_FIFO_TX1_8, UART_FIFO_RX4_8);
        if (isFlowControlEnabled())
        {
            UARTHwFlowControlEnable(obj_.base);
        }
        else
        {
            UARTHwFlowControlDisable(obj_.base);
        }
        IOCPinTypeUart(obj_.base,
            obj_.pins.rx,
            obj_.pins.tx,
            obj_.pins.cts,
            obj_.pins.rts
        );
        UARTEnable(obj_.base);

        // Get read of residual data from the RX FIFO
        flush();
    }

    ~Uart()
    {
    }

    bool read(uint8_t* buf, size_t len)
    {
        while (len > 0)
        {
          int32_t ch = UARTCharGet(obj_.base);
          *buf++ = (uint8_t)ch;

          len--;
        }

        return true;
    }

    bool write(const uint8_t* buf, size_t len)
    {
        while (len > 0)
        {
            UARTCharPut(obj_.base, *buf++);

            len--;
        }

        while (UARTBusy(obj_.base));

        return true;
    }

    void flush()
    {
        while (UARTCharGetNonBlocking(obj_.base) != -1);
    }

private:
    bool isFlowControlEnabled()
    {
        return (obj_.pins.cts != IOID_UNUSED) && (obj_.pins.rts != IOID_UNUSED);
    }
};

} /* namespace bsp */

#endif /* UART_HPP_ */
