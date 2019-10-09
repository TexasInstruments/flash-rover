
#ifndef SPI_HPP_
#define SPI_HPP_

#include <stddef.h>
#include <stdint.h>

#include "power.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/ioc.h)
#include DeviceFamily_constructPath(driverlib/ssi.h)
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
            if (!SSIDataPutNonBlocking(obj_.base, 0))
            {
                /* Error */
                return false;
            }

            uint32_t ul;
            SSIDataGet(obj_.base, &ul);
            *buf++ = (uint8_t)ul;

            len--;
        }

        return true;
    }

    bool write(const uint8_t* buf, size_t len)
    {
        while (len > 0)
        {
            SSIDataPut(obj_.base, *buf++);

            uint32_t dummy;
            SSIDataGet(obj_.base, &dummy);

            len--;
        }

        return true;
    }

    void flush()
    {
        uint32_t dummy;
        while (SSIDataGetNonBlocking(obj_.base, &dummy));
    }
};

} /* namespace bsp */

#endif /* SPI_HPP_ */
