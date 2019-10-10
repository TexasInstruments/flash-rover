
#include <stdint.h>

#include <limits>

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/interrupt.h)

#include "bsp/conf.hpp"
#include "bsp/doorbell.hpp"
#include "bsp/ext_flash.hpp"
#include "bsp/power.hpp"
#include "bsp/spi.hpp"
#include "hard_fault.hpp"

using namespace bsp;

RegDump_t regDump;

__attribute__((section (".conf")))
volatile const Conf conf;

__attribute__((section (".doorbell")))
volatile Doorbell doorbell;

#define XFLASH_BUF_SIZE  0x1000

__attribute__((section (".xflashbuf")))
uint8_t xflashbuf[XFLASH_BUF_SIZE];

class Loop
{
private:
    Spi     spi_;
    Xflash  xflash_;
    Server  server_;

public:
    Loop(Power& power, const SpiObj& spiObj, const XflashObj& xflashObj)
        : spi_{ spiObj, power }
        , xflash_{ xflashObj, spi_, power }
        , server_{ doorbell }
    {
    }

    ~Loop()
    {
    }

    void run()
    {
        while (true)
        {
            auto cmd = server_.waitForCommand();
            Response rsp;

            switch (cmd.kind)
            {
            case Command::Kind::XflashInfo:  rsp = xflashInfo(cmd);  break;
            case Command::Kind::MassErase:   rsp = massErase(cmd);   break;
            case Command::Kind::SectorErase: rsp = sectorErase(cmd); break;
            case Command::Kind::ReadBlock:   rsp = readBlock(cmd);   break;
            case Command::Kind::WriteBlock:  rsp = writeBlock(cmd);  break;
            default:                         rsp = error();          break;
            }

            server_.sendResponse(rsp);
        }
    }

private:
    Response xflashInfo(const Command&)
    {
        const auto* maybe_info = xflash_.getInfo();
        if (maybe_info == nullptr)
        {
            return error(Response::Kind::ErrorXflash);
        }

        const auto& info = *maybe_info;

        return {
            Response::Kind::XflashInfo,
            info.manfId,
            info.devId
        };
    }

    Response massErase(const Command&)
    {
        bool ret = xflash_.massErase();

        if (ret)
        {
            return {
                Response::Kind::Ok
            };
        }
        else
        {
            return error(Response::Kind::ErrorXflash);
        }
    }

    Response sectorErase(const Command& cmd)
    {
        uint32_t offset = cmd.arg0;
        uint32_t length = cmd.arg1;

        bool ret = xflash_.erase(length, offset);

        if (ret)
        {
            return {
                Response::Kind::Ok
            };
        }
        else
        {
            return error(Response::Kind::ErrorXflash);
        }
    }

    Response readBlock(const Command& cmd)
    {
        uint32_t offset = cmd.arg0;
        uint32_t length = cmd.arg1;

        if (length > XFLASH_BUF_SIZE)
        {
            return error(Response::Kind::ErrorBufOverflow);
        }


        bool ret = xflash_.read(xflashbuf, length, offset);

        if (ret)
        {
            return {
                Response::Kind::Ok
            };
        }
        else
        {
            return error(Response::Kind::ErrorXflash);
        }
    }

    Response writeBlock(const Command& cmd)
    {
        uint32_t offset = cmd.arg0;
        uint32_t length = cmd.arg1;

        if (length > XFLASH_BUF_SIZE)
        {
            return error(Response::Kind::ErrorBufOverflow);
        }

        bool ret = xflash_.write(xflashbuf, length, offset);

        if (ret)
        {
            return {
                Response::Kind::Ok
            };
        }
        else
        {
            return error(Response::Kind::ErrorXflash);
        }
    }

    Response error(Response::Kind kind = Response::Kind::Error)
    {
        return { kind };
    }
};

void loop()
{
    SpiObj spiObj = defaultSpiObj;
    XflashObj xflashObj = defaultXflashObj;

    if (conf.valid != 0)
    {
        // Note that CSN is software controlled, hence we set it in the
        // xflashObj. Setting CSN in spiObj would make it HW controlled.
        spiObj.pins.miso = conf.spiPins.miso;
        spiObj.pins.mosi = conf.spiPins.mosi;
        spiObj.pins.clk = conf.spiPins.clk;
        xflashObj.csn = conf.spiPins.csn;
    }

    Power power;
    Loop loop{ power, spiObj, xflashObj };
    loop.run();
}

int main()
{
    IntMasterEnable();

//#ifndef MAKE_FW
    openHardFaultDebugger(regDump);
//#endif

    loop();

    for (;;);
}
