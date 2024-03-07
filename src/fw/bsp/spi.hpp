// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

#ifndef SPI_HPP_
#define SPI_HPP_

#include <stddef.h>
#include <stdint.h>

#include "power.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/ioc.h)
#ifdef DeviceFamily_CC13X4
#include DeviceFamily_constructPath(driverlib/spi.h)
#else
#include DeviceFamily_constructPath(driverlib/ssi.h)
#endif
#include DeviceFamily_constructPath(inc/hw_memmap.h)

namespace bsp {

struct SpiPins
{
    uint32_t miso{ IOID_UNUSED };
    uint32_t mosi{ IOID_UNUSED };
    uint32_t clk{ IOID_UNUSED };
    uint32_t csn{ IOID_UNUSED };
} __attribute__((packed));

struct SpiObj
{
    Power::Periph periph{ Power::Periph::None };
    uint32_t base{ 0xFFFFFFFF };
    SpiPins pins{};
};

#ifdef DeviceFamily_CC13X4
constexpr const SpiObj defaultSpiObj = {
    Power::Periph::Ssi0,  /* periph */
    SPI0_BASE,            /* base */
    {                     /* pins */
         IOID_37,         /* poci */
         IOID_36,         /* pico */
         IOID_39,         /* clk */
         IOID_UNUSED,     /* csn */
    },
};
#else
constexpr const SpiObj defaultSpiObj = {
    Power::Periph::Ssi0,  /* periph */
    SSI0_BASE,            /* base */
    {                     /* pins */
         IOID_8,          /* miso */
         IOID_9,          /* mosi */
         IOID_10,         /* clk */
         IOID_UNUSED,     /* csn */
    },
};
#endif

class Spi
{
private:
    SpiObj              obj_;
    Power::PeriphHandle periph_;

public:
    Spi(const SpiObj& obj, Power& power)
        : obj_{ obj }
        , periph_{ power.openPeriph(obj_.periph) }
    {
        #ifdef DeviceFamily_CC13X4
        SPIIntDisable(obj_.base, SPI_MIS_RXFIFO_OVF_SET | SPI_MIS_PER_SET | SPI_MIS_TX_SET | SPI_MIS_RTOUT_SET);
        SPIIntClear(obj_.base, SPI_MIS_RXFIFO_OVF_SET | SPI_MIS_PER_SET);
        SPIConfigSetExpClk(obj_.base,
            48000000,             /* CPU rate */
            SPI_FRF_MOTO_MODE_0,  /* frame format */
            SPI_MODE_CONTROLLER,  /* mode */
            4000000,              /* bit rate */
            8                     /* data size */
        );
        IOCPinTypeSpiMaster(obj_.base,
            obj_.pins.miso,
            obj_.pins.mosi,
            obj_.pins.csn,
            obj_.pins.clk
        );
        SPIEnable(obj_.base);
        #else
        SSIIntDisable(obj_.base, SSI_RXOR | SSI_RXFF | SSI_RXTO | SSI_TXFF);
        SSIIntClear(obj_.base, SSI_RXOR | SSI_RXTO);
        SSIConfigSetExpClk(obj_.base,
            48000000,             /* CPU rate */
            SSI_FRF_MOTO_MODE_0,  /* frame format */
            SSI_MODE_MASTER,      /* mode */
            4000000,              /* bit rate */
            8                     /* data size */
        );
        IOCPinTypeSsiMaster(obj_.base,
            obj_.pins.miso,
            obj_.pins.mosi,
            obj_.pins.csn,
            obj_.pins.clk
        );
        SSIEnable(obj_.base);
        #endif
        // Get read of residual data from SSI port
        flush();
    }

    ~Spi()
    {
    }

    bool read(uint8_t* buf, size_t len)
    {
        while (len > 0)
        {
            #ifdef DeviceFamily_CC13X4
            if (!SPIDataPutNonBlocking(obj_.base, 0))
            {
                /* Error */
                return false;
            }

            uint32_t ul;
            SPIDataGet(obj_.base, &ul);
            #else
            if (!SSIDataPutNonBlocking(obj_.base, 0))
            {
                /* Error */
                return false;
            }

            uint32_t ul;
            SSIDataGet(obj_.base, &ul);
            #endif

            *buf++ = (uint8_t)ul;

            len--;
        }

        return true;
    }

    bool write(const uint8_t* buf, size_t len)
    {
        while (len > 0)
        {
            #ifdef DeviceFamily_CC13X4
            SPIDataPut(obj_.base, *buf++);

            uint32_t dummy;
            SPIDataGet(obj_.base, &dummy);
            #else
            SSIDataPut(obj_.base, *buf++);

            uint32_t dummy;
            SSIDataGet(obj_.base, &dummy);
            #endif

            len--;
        }

        return true;
    }

    void flush()
    {
        uint32_t dummy;

        #ifdef DeviceFamily_CC13X4
        while (SPIDataGetNonBlocking(obj_.base, &dummy));
        #else
        while (SSIDataGetNonBlocking(obj_.base, &dummy));
        #endif
    }
};

} /* namespace bsp */

#endif /* SPI_HPP_ */
