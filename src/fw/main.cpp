
#include <string.h>

#include <limits>

#include "spi.hpp"
#include "power.hpp"
#include "uart.hpp"
#include "ext_flash.hpp"
#include "serialize.hpp"
#include "hard_fault.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/interrupt.h)

using namespace bsp;


RegDump_t regDump;


class Loop
{
private:
    Power&    power_;
    Uart      uart_;
    Serialize serialize_;
    Spi       spi_;
    ExtFlash  extFlash_;

    uint8_t extFlashBuf_[ExtFlash::programPageSize];

public:
    Loop(Power& power, UartObj uartObj, SpiObj spiObj)
        : power_{ power }
        , uart_{ uartObj, power_ }
        , serialize_{ uart_ }
        , spi_{ spiObj, power_ }
        , extFlash_{ extFlashLp, spi_, power_ }
    {
    }

    ~Loop()
    {
    }

    void run()
    {
        while (true)
        {
            auto cmd = serialize_.readCmd(extFlashBuf_, sizeof(extFlashBuf_));

            switch (cmd.type)
            {
            case Serialize::Cmd::Type::Sync:
                sync(cmd);
                break;

            case Serialize::Cmd::Type::FlashInfo:
                flashInfo(cmd);
                break;

            case Serialize::Cmd::Type::MassErase:
            case Serialize::Cmd::Type::Erase:
                erase(cmd);
                break;

            case Serialize::Cmd::Type::Read:
                read(cmd);
                break;

            case Serialize::Cmd::Type::StartWrite:
                startWrite();
                break;

            case Serialize::Cmd::Type::DataWrite:
                dataWrite(cmd);
                break;

            default:
                sendError();
                break;
            }
        }
    }

private:
    void sync(Serialize::Cmd)
    {
        sendAck();
    }

    void flashInfo(Serialize::Cmd)
    {
        auto info = extFlash_.getInfo();
        if (info.isLeft)
        {
            sendFlashInfo(info.leftValue);
        }
        else
        {
            sendError(Serialize::Response::Type::ErrorExtFlash);
        }
    }

    void erase(Serialize::Cmd cmd)
    {
        sendAckPend();

        bool ret;
        if (cmd.type == Serialize::Cmd::Type::MassErase)
        {
            ret = extFlash_.massErase();
        }
        else
        {
            uint32_t offset = cmd.arg0;
            uint32_t length = cmd.arg1;

            if (!checkAddressRange(offset, length))
            {
                sendError(Serialize::Response::Type::ErrorAddressRange);
                return;
            }

            ret = extFlash_.erase(length, offset);
        }

        if (ret)
        {
            sendAck();
        }
        else
        {
            sendError(Serialize::Response::Type::ErrorExtFlash);
        }
    }

    void read(Serialize::Cmd cmd)
    {
        sendAckPend();

        uint32_t offset = cmd.arg0;
        uint32_t length = cmd.arg1;

        if (!checkAddressRange(offset, length))
        {
            sendError(Serialize::Response::Type::ErrorAddressRange);
            return;
        }

        bool ret;
        while (length > 0)
        {
            uint32_t ilength = std::min(length, sizeof(extFlashBuf_));

            ret = extFlash_.read(extFlashBuf_, ilength, offset);
            if (!ret)
            {
                sendError(Serialize::Response::Type::ErrorExtFlash);
                return;
            }

            sendData(extFlashBuf_, ilength, offset);

            offset += ilength;
            length -= ilength;
        }

        sendAck();
    }

    void startWrite()
    {
        sendWriteSize();
    }

    void dataWrite(Serialize::Cmd cmd)
    {
        sendAckPend();

        uint32_t offset = cmd.arg0;
        uint32_t length = cmd.arg1;

        if (length > sizeof(extFlashBuf_))
        {
            sendError(Serialize::Response::Type::ErrorBufferOverflow);
            return;
        }

        if (!checkAddressRange(offset, length))
        {
            sendError(Serialize::Response::Type::ErrorAddressRange);
            return;
        }

        bool ret = extFlash_.write(extFlashBuf_, length, offset);
        if (!ret)
        {
            sendError(Serialize::Response::Type::ErrorExtFlash);
            return;
        }

        sendAck();
    }

    bool checkAddressRange(uint32_t offset, uint32_t length) const
    {
        uint64_t endPoint = static_cast<uint64_t>(offset) + static_cast<uint64_t>(length);
        if (endPoint > static_cast<uint64_t>(std::numeric_limits<uint32_t>::max()))
        {
            return false;
        }

        auto info = extFlash_.getInfo();
        if (!info.isLeft)
        {
            return false;
        }

        uint64_t deviceSize = static_cast<uint64_t>(info.leftValue.deviceSize);
        if (info.leftValue.supported && endPoint > deviceSize)
        {
            return false;
        }

        return true;
    }

    void sendAck()
    {
        serialize_.sendResponse(Serialize::Response{
            Serialize::Response::Type::Ack
        });
    }

    void sendAckPend()
    {
        serialize_.sendResponse(Serialize::Response{
            Serialize::Response::Type::AckPend
        });
    }

    void sendFlashInfo(const ExtFlashInfo& info)
    {
        if (info.supported)
        {
            serialize_.sendResponse(Serialize::Response{
                Serialize::Response::Type::FlashInfo,
                info.manfId,
                info.devId,
                info.deviceSize
            });
        }
        else
        {
            serialize_.sendResponse(Serialize::Response{
                Serialize::Response::Type::ErrorUnsupported,
                info.manfId,
                info.devId
            });
        }
    }

    void sendData(const uint8_t* buf, uint32_t length, uint32_t offset)
    {
        serialize_.sendResponse(Serialize::Response{
            Serialize::Response::Type::DataRead,
            offset,
            length
        }, buf, length);
    }

    void sendWriteSize()
    {
        serialize_.sendResponse(Serialize::Response{
            Serialize::Response::Type::WriteSize,
            sizeof(extFlashBuf_)
        });
    }

    void sendError(Serialize::Response::Type type = Serialize::Response::Type::Error)
    {
        serialize_.sendResponse(Serialize::Response{
            type
        });
    }
};


int main(void)
{
    IntMasterEnable();

    //openHardFaultDebugger(regDump);

    Power power;
    Loop loop{ power, uart0Obj, spi0Obj };
    loop.run();

    for (;;);
}
