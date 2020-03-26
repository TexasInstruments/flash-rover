// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

#ifndef EXT_FLASH_HPP_
#define EXT_FLASH_HPP_

#include <stdlib.h>
#include <stdbool.h>

#include <array>

#include "power.hpp"
#include "spi.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/ioc.h)
#include DeviceFamily_constructPath(driverlib/gpio.h)
#include DeviceFamily_constructPath(driverlib/cpu.h)

namespace bsp {

enum class XflashError
{
    Generic,
    Unsupported,
};

struct XflashInfo
{
    uint32_t deviceSize{ 0 };
    uint8_t manfId{ 0 };
    uint8_t devId{ 0 };
    bool supported{ false };
};

static constexpr std::array<XflashInfo, 4> supportedHw = {{
    // Macronics MX25R1635F
    {
        0x200000,  // 2 MByte (16 Mbit)
        0xC2,
        0x15
    },
    // Macronics MX25R8035F
    {
        0x100000,  // 1 MByte (8 Mbit)
        0xC2,
        0x14
    },
    // WinBond W25X40CL
    {
        0x080000,  // 512 KByte (4 Mbit)
        0xEF,
        0x12
    },
    // WinBond W25X20CL
    {
        0x040000,  // 256 KByte (2 Mbit)
        0xEF,
        0x11
    },
}};

struct XflashObj
{
    uint32_t csn{ IOID_UNUSED };
};

constexpr const XflashObj defaultXflashObj = {
    IOID_20,  /* csn */
};

class Xflash
{
private:
    struct OpCode
    {
        static constexpr uint8_t program      = 0x02;  // Page program
        static constexpr uint8_t read         = 0x03;  // Read data
        static constexpr uint8_t read_status  = 0x05;  // Read status register
        static constexpr uint8_t write_enable = 0x06;  // Write enable
        static constexpr uint8_t erase_4k     = 0x20;  // Sector erase 4K bytes
        static constexpr uint8_t erase_32k    = 0x52;  // Sector erase 32K bytes
        static constexpr uint8_t erase_64k    = 0xD8;  // Sector erase 64K bytes
        static constexpr uint8_t erase_all    = 0xC7;  // Sector erase all bytes
        static constexpr uint8_t mdid         = 0x90;  // Manufacturer Device ID
        static constexpr uint8_t dp           = 0xB9;  // Power down
        static constexpr uint8_t rdp          = 0xAB;  // Power standby
        static constexpr uint8_t rsten        = 0x66;  // Reset-Enable
        static constexpr uint8_t rst          = 0x99;  // Reset
    };

    struct StatusCode
    {
        uint8_t wip:1;  // bit0: Write in progress
        uint8_t wel:1;  // bit1: Write enable latch
        uint8_t bp0:1;  // bit2: Level of protected block
        uint8_t bp1:1;  // bit3: Level of protected block
        uint8_t bp2:1;  // bit4: Level of protected block
        uint8_t bp3:1;  // bit5: Level of protected block
        uint8_t qe:1;   // bit6: Quad enabled
        uint8_t srwd:1; // bit7: Status register write protect
    };

    static_assert(sizeof(StatusCode) == 1);

    XflashObj           obj_;
    Power::PeriphHandle gpioPeriph_;
    Spi&                spi_;
    struct
    {
        XflashInfo info{};
        bool valid{ false };
    } xflash_;

public:
    static constexpr uint32_t programPageSize = 256;
    static constexpr uint32_t eraseSectorSize = 4096;

    Xflash(const XflashObj& obj, Spi& spi, Power& power)
        : obj_{ obj }
        , gpioPeriph_{ power.openPeriph(Power::Periph::Gpio) }
        , spi_{ spi }
    {
        IOCPinTypeGpioOutput(obj_.csn);

        deselect();

        if (!powerStandby())
        {
            close();
            return;
        }

        if (!verifyPart())
        {
            close();
        }
    }

    ~Xflash()
    {
        close();
    }

    const XflashInfo* getInfo() const
    {
        if (xflash_.valid)
        {
            return &xflash_.info;
        }
        else
        {
            return nullptr;
        }
    }

    bool read(uint8_t* buf, size_t len, size_t offset)
    {
        // Wait till previous erase/program operation completes
        bool ret = waitReady();
        if (!ret)
        {
            return false;
        }

        // SPI is driven with very low frequency (1MHz < 33MHz fR spec)
        // in this temporary implementation.
        // and hence it is not necessary to use fast read.
        const uint8_t wbuf[] = {
            OpCode::read,
            static_cast<uint8_t>(offset >> 16),
            static_cast<uint8_t>(offset >> 8),
            static_cast<uint8_t>(offset),
        };

        select();

        ret = spi_.write(wbuf, sizeof(wbuf));
        if (ret)
        {
            ret = spi_.read(buf, len);
        }

        deselect();

        return ret;
    }

    bool write(const uint8_t* buf, size_t len, size_t offset)
    {
        bool ret;

        while (len > 0)
        {
            // Wait till previous erase/program operation completes
            ret = waitReady();
            if (!ret)
            {
                return false;
            }

            ret = writeEnable();
            if (!ret)
            {
                return false;
            }

            // interim length per instruction
            size_t ilen = programPageSize - (offset % programPageSize);
            if (len < ilen)
            {
                ilen = len;
            }

            const uint8_t wbuf[] = {
                OpCode::program,
                static_cast<uint8_t>(offset >> 16),
                static_cast<uint8_t>(offset >> 8),
                static_cast<uint8_t>(offset),
            };

            offset += ilen;
            len -= ilen;

            // Up to 100ns CS hold time (which is not clear
            // whether it's application only in between reads)
            // is not imposed here since above instructions
            // should be enough to delay
            // as much.
            select();

            ret = spi_.write(wbuf, sizeof(wbuf));
            if (ret)
            {
                ret = spi_.write(buf, ilen);
            }

            deselect();

            if (!ret)
            {
                return false;
            }

            buf += ilen;
        }

        return true;
    }

    bool erase(size_t len, size_t offset)
    {
        // Note that Block erase might be more efficient when the floor map
        // is well planned for OTA but to simplify for the temporary implementation,
        // sector erase is used blindly.
        size_t endoffset = offset + len - 1;
        offset = (offset / eraseSectorSize) * eraseSectorSize;
        size_t numsectors = (endoffset - offset + eraseSectorSize - 1) / eraseSectorSize;

        bool ret;

        for (size_t i = 0; i < numsectors; i++)
        {
            // Wait till previous erase/program operation completes
            ret = waitReady();
            if (!ret)
            {
                return false;
            }

            ret = writeEnable();
            if (!ret)
            {
                return false;
            }

            const uint8_t wbuf[] = {
                OpCode::erase_4k,
                static_cast<uint8_t>(offset >> 16),
                static_cast<uint8_t>(offset >> 8),
                static_cast<uint8_t>(offset),
            };

            select();

            ret = spi_.write(wbuf, sizeof(wbuf));

            deselect();

            if (!ret)
            {
                return false;
            }

            offset += eraseSectorSize;
        }

        return waitReady();
    }

    bool massErase()
    {
        bool ret;

        // Wait till previous erase/program operation completes
        ret = waitReady();
        if (!ret)
        {
            return false;
        }

        ret = writeEnable();
        if (!ret)
        {
            return false;
        }

        const uint8_t wbuf[] = { OpCode::erase_all };

        select();

        ret = spi_.write(wbuf, sizeof(wbuf));

        deselect();

        return waitReady();
    }

    bool reset()
    {
        bool ret;

        ret = waitReady();
        if (!ret)
        {
            return false;
        }

        const uint8_t rsten_buf[] = { OpCode::rsten };
        const uint8_t rst_buf[] = { OpCode::rst };


        select();

        ret = spi_.write(rsten_buf, sizeof(rsten_buf));

        deselect();

        if (!ret)
        {
            return false;
        }


        // Wait for at least 1 us.
        Xflash::delay(1);

        select();

        ret = spi_.write(rst_buf, sizeof(rst_buf));

        deselect();

        if (!ret)
        {
            return false;
        }

        // Wait for at least 20 ms.
        Xflash::delay(20 * 1000);

        waitPowerDown();

        ret = powerStandby();
        if (!ret)
        {
            return false;
        }

        return waitReady();
    }

    void close()
    {
        // Put the part in low power mode
        powerDown();
        waitPowerDown();

    }

private:
    static void delay(uint32_t us)
    {
        // ui32Count = [delay in us] * [CPU clock in MHz] / [cycles per loop]
        CPUdelay((us * 48) / 4);
    }

    void select()
    {
        GPIO_clearDio(obj_.csn);
    }

    void deselect()
    {
        GPIO_setDio(obj_.csn);
    }

    bool readInfo()
    {
        const uint8_t wbuf[] = { OpCode::mdid, 0xFF, 0xFF, 0x00 };
        uint8_t rbuf[2];

        select();

        bool ret = spi_.write(wbuf, sizeof(wbuf));
        if (ret)
        {
            ret = spi_.read(rbuf, sizeof(rbuf));
            if (ret)
            {
                xflash_.info.manfId = rbuf[0];
                xflash_.info.devId = rbuf[1];
            }
        }
        xflash_.valid = ret;

        deselect();

        return ret;
    }

    bool verifyPart()
    {
        if (!readInfo())
        {
            return false;
        }

        if (!xflash_.valid)
        {
            return false;
        }

        for (const XflashInfo& hw : supportedHw)
        {
            if (xflash_.info.manfId == hw.manfId && xflash_.info.devId == hw.devId)
            {
                xflash_.info.supported = true;
                xflash_.info.deviceSize = hw.deviceSize;
                return true;
            }
        }

        return false;
    }

    bool powerDown()
    {
        const uint8_t wbuf[] = { OpCode::dp };

        select();

        bool ret = spi_.write(wbuf, sizeof(wbuf));

        deselect();

        return ret;
    }

    bool powerStandby()
    {
        const uint8_t wbuf[] = { OpCode::rdp };

        select();

        bool success = spi_.write(wbuf, sizeof(wbuf));

        deselect();

        if (!success)
        {
            return false;
        }

        // Waking up of the device is manufacturer dependent.
        // for a Winond chip-set, once the request to wake up the flash has been
        // send, CS needs to stay high at least 3us (for Winbond part)
        // for chip-set like Macronix, it can take up to 35us.
        // Sleeping for 100 us should give enough margin.
        Xflash::delay(100);

        return waitReady();
    }

    bool waitReady()
    {
        const uint8_t wbuf[1] = { OpCode::read_status };

        /* Throw away garbage */
        select();
        spi_.flush();
        deselect();

        for (;;)
        {
            uint8_t rbuf;

            select();

            spi_.write(wbuf, sizeof(wbuf));
            bool ret = spi_.read(&rbuf, sizeof(rbuf));

            deselect();

            if (!ret)
            {
                return false;
            }

            StatusCode status_code = *reinterpret_cast<StatusCode *>(&rbuf);

            // Xfash is not busy if work-in-progress bit is not set
            if (status_code.wip == 0)
            {
                /* Now ready */
                return true;
            }
        }
    }

    bool waitPowerDown()
    {
        for (int i = 0; i < 10; ++i)
        {
            if (!verifyPart())
            {
                return true;
            }
        }

        return false;
    }

    bool writeEnable()
    {
        const uint8_t wbuf[] = { OpCode::write_enable };

        select();

        bool ret = spi_.write(wbuf, sizeof(wbuf));

        deselect();

        return ret;
    }
};

} /* namespace bsp */

#endif /* EXT_FLASH_HPP_ */
