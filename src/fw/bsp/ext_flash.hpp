
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

#include "neither/neither.hpp"

namespace bsp {

#define EXT_FLASH_PAGE_SIZE   4096

enum class ExtFlashError
{
    Generic,
    Unsupported,
};

struct ExtFlashInfo
{
    uint32_t deviceSize{ 0 };
    uint8_t manfId{ 0 };
    uint8_t devId{ 0 };
    bool supported{ false };
};

static constexpr std::array<ExtFlashInfo, 4> supportedHw = {{
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

struct ExtFlashObj
{
    uint32_t csn{ IOID_UNUSED };
};

constexpr const ExtFlashObj extFlashLp = {
    IOID_20,  /* csn */
};

class ExtFlash
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
    };

    struct StatusCode
    {
        static constexpr uint8_t SRWD = 0x80;
        static constexpr uint8_t BP   = 0x0C;
        static constexpr uint8_t WEL  = 0x02;
        static constexpr uint8_t BUSY = 0x01;
    };

    ExtFlashObj         obj_;
    Power::PeriphHandle gpioPeriph_;
    Spi&                spi_;
    struct
    {
        ExtFlashInfo info{};
        bool valid{ false };
    } extFlash_;

public:
    static constexpr uint32_t programPageSize = 256;
    static constexpr uint32_t eraseSectorSize = 4096;

    ExtFlash(const ExtFlashObj& obj, Spi& spi, Power& power)
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

    ~ExtFlash()
    {
        close();
    }

    neither::Either<ExtFlashInfo, ExtFlashError> getInfo() const
    {
        if (extFlash_.valid)
        {
            return neither::left(extFlash_.info);
        }
        else
        {
            return neither::right(ExtFlashError::Generic);
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

    void close()
    {
        // Put the part in low power mode
        powerDown();
        waitPowerDown();
    }

private:
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
                extFlash_.info.manfId = rbuf[0];
                extFlash_.info.devId = rbuf[1];
            }
        }
        extFlash_.valid = ret;

        deselect();

        return ret;
    }

    bool verifyPart()
    {
        if (!readInfo())
        {
            return false;
        }

        if (!extFlash_.valid)
        {
            return false;
        }

        for (const ExtFlashInfo& hw : supportedHw)
        {
            if (extFlash_.info.manfId == hw.manfId && extFlash_.info.devId == hw.devId)
            {
                extFlash_.info.supported = true;
                extFlash_.info.deviceSize = hw.deviceSize;
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
        // ui32Count = [delay in us] * [CPU clock in MHz] / [cycles per loop]
        CPUdelay((35 * 48) / 4);

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

            if ((rbuf & StatusCode::BUSY) == 0)
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
