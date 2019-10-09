
#ifndef SERIALIZE_HPP_
#define SERIALIZE_HPP_

#include <stdint.h>
#include <string.h>

#include <algorithm>

#include "uart.hpp"

namespace bsp {

class Serialize
{
private:
    Uart& uart_;

    static constexpr uint8_t startOp = 0xEF;
public:
    //
    // Packet format
    //
    //   StartOp   Type       Arg(s) [u32]   Data [8]
    // +---------+----------+--------------+------------+
    // |    EF   |  <Type>  |   .. (N) ..  |  .. (N) .. |
    // +---------+----------+--------------+------------+
    //

    struct Cmd
    {
        enum class Type : uint8_t
        {
            Invalid    = 0x00,

            Sync       = 0xC0,
            FlashInfo  = 0xC1,
            Erase      = 0xC2,  // <offset (u32), length (u32)>
            MassErase  = 0xC3,
            Read       = 0xC4,  // <offset (u32), length (u32)>
            StartWrite = 0xC5,
            DataWrite  = 0xC6,  // <offset (u32), length (u32), data...(u8)>
        };

        Type type;
        uint32_t arg0;
        uint32_t arg1;
    };

    struct Response
    {
        enum class Type : uint8_t
        {
            Invalid             = 0x00,

            Ack                 = 0x01,
            AckPend             = 0x02,
            FlashInfo           = 0x03,  // <manfId (u8), devId (u8), devSize (u32)>
            WriteSize           = 0x04,  // <length (u32)>
            DataRead            = 0x05,  // <offset (u32), length (u32), data... (u8)>

            Error               = 0x80,
            ErrorExtFlash       = 0x81,
            ErrorUnsupported    = 0x82,  // <manfId, devId>
            ErrorAddressRange   = 0x83,
            ErrorBufferOverflow = 0x83,
        };

        Type type{ Type::Invalid };
        uint32_t arg0{ 0 };
        uint32_t arg1{ 0 };
        uint32_t arg2{ 0 };
    };



    Serialize(Uart& uart)
        : uart_{ uart }
    {
    }

    ~Serialize()
    {
    }

    Cmd readCmd(uint8_t* buf, size_t len)
    {
        Cmd cmd;

        bool ret;

        while (true)
        {
            memset(&cmd, 0, sizeof(Cmd));

            uint8_t ch;
            do
            {
                ret = uart_.read(&ch, 1);
            } while (!(ret && ch == startOp));
            ret = uart_.read(&ch, 1);
            if (!ret)
            {
                // Failed uart, try again
                continue;
            }

            cmd.type = static_cast<Cmd::Type>(ch);

            switch (cmd.type)
            {
            case Cmd::Type::Erase:
            case Cmd::Type::Read:
                ret =        uart_.read(reinterpret_cast<uint8_t *>(&cmd.arg0), sizeof(uint32_t));
                ret = ret && uart_.read(reinterpret_cast<uint8_t *>(&cmd.arg1), sizeof(uint32_t));
                uart_.flush();
                if (!ret)
                {
                    // Failed uart, try again
                    continue;
                }
                return cmd;

            case Cmd::Type::DataWrite:
                ret =        uart_.read(reinterpret_cast<uint8_t *>(&cmd.arg0), sizeof(uint32_t));
                ret = ret && uart_.read(reinterpret_cast<uint8_t *>(&cmd.arg1), sizeof(uint32_t));
                ret = ret && uart_.read(buf, std::min((size_t)cmd.arg1, len));
                uart_.flush();
                if (!ret)
                {
                    // Failed uart, try again
                    continue;
                }
                return cmd;

            case Cmd::Type::Sync:
            case Cmd::Type::FlashInfo:
            case Cmd::Type::MassErase:
            case Cmd::Type::StartWrite:
                uart_.flush();
                return cmd;

            default:
                break;
            }
        }
    }

    void sendResponse(const Response& rsp, const uint8_t* buf, size_t len)
    {
        const uint8_t startOpCh = startOp;
        const uint8_t typeCh = static_cast<uint8_t>(rsp.type);

        bool ret = true;
        ret = ret && uart_.write(&startOpCh, 1);
        ret = ret && uart_.write(&typeCh, 1);

        switch (rsp.type)
        {
        case Response::Type::FlashInfo:
            ret = ret && uart_.write(reinterpret_cast<const uint8_t*>(&rsp.arg0), sizeof(uint8_t));
            ret = ret && uart_.write(reinterpret_cast<const uint8_t *>(&rsp.arg1), sizeof(uint8_t));
            ret = ret && uart_.write(reinterpret_cast<const uint8_t *>(&rsp.arg2), sizeof(uint32_t));
            break;

        case Response::Type::DataRead:
            ret = ret && uart_.write(reinterpret_cast<const uint8_t*>(&rsp.arg0), sizeof(uint32_t));
            ret = ret && uart_.write(reinterpret_cast<const uint8_t *>(&len), sizeof(uint32_t));
            ret = ret && uart_.write(buf, len);
            break;

        case Response::Type::WriteSize:
            ret = ret && uart_.write(reinterpret_cast<const uint8_t*>(&rsp.arg0), sizeof(uint32_t));
            break;

        case Response::Type::Ack:
        case Response::Type::AckPend:
        default:
            break;
        }
    }

    void sendResponse(const Response& rsp)
    {
        sendResponse(rsp, nullptr, 0);
    }
};

} /* namespace bsp */

#endif /* SERIALIZE_HPP_ */
