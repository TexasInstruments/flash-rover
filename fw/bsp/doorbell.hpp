// This file is covered by the LICENSE file in the root of this project.

#ifndef DOORBELL_HPP_
#define DOORBELL_HPP_

#include <stdint.h>
#include <string.h>

#include <algorithm>

#include "spi.hpp"

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/ioc.h)

namespace bsp {

#define ATTR_PACKED  __attribute__((packed))

struct Command
{
    enum class Kind : uint32_t
    {
        None        = 0x00,

        XflashInfo  = 0xC0,
        SectorErase = 0xC1,  // <offset (u32), length (u32)>
        MassErase   = 0xC2,
        ReadBlock   = 0xC3,  // <offset (u32), length (u32)>
        WriteBlock  = 0xC4,  // <offset (u32), length (u32)> <data... (u8)>
    };

    Kind kind{ Kind::None };
    uint32_t arg0{ 0 };
    uint32_t arg1{ 0 };
    uint32_t arg2{ 0 };
} ATTR_PACKED;

struct Response
{
    enum class Kind : uint32_t
    {
        None             = 0x00,

        Ok               = 0xD0,
        XflashInfo       = 0xD1,  // <manfId (u8), devId (u8)>

        Error            = 0x80,
        ErrorSpi         = 0x81,
        ErrorXflash      = 0x82,
        ErrorBufOverflow = 0x83,
    };

    Kind kind{ Kind::None };
    uint32_t arg0{ 0 };
    uint32_t arg1{ 0 };
    uint32_t arg2{ 0 };
} ATTR_PACKED;



struct Doorbell
{
    Command cmd;
    Response rsp;
} ATTR_PACKED;

class Server
{
private:
    volatile Doorbell& doorbell_;

public:
    Server(volatile Doorbell& doorbell)
        : doorbell_{ doorbell }
    {
        doorbell_.cmd.kind = Command::Kind::None;
        doorbell_.rsp.kind = Response::Kind::None;
    }

    ~Server()
    {
    }

    Command waitForCommand()
    {
        Command cmd{};
        for (;;)
        {
            while (doorbell_.cmd.kind == Command::Kind::None);

            switch (doorbell_.cmd.kind)
            {
            case Command::Kind::XflashInfo:
            case Command::Kind::MassErase:
            case Command::Kind::SectorErase:
            case Command::Kind::ReadBlock:
            case Command::Kind::WriteBlock:
                cmd.kind = doorbell_.cmd.kind;
                cmd.arg0 = doorbell_.cmd.arg0;
                cmd.arg1 = doorbell_.cmd.arg1;
                cmd.arg2 = doorbell_.cmd.arg2;

                doorbell_.cmd.kind = Command::Kind::None;
                return cmd;

            default:
                // Invalid command, clear command and wait again.
                doorbell_.cmd.kind = Command::Kind::None;
                continue;
            }
        }
    }

    void sendResponse(const Response& rsp)
    {
        doorbell_.rsp.arg0 = rsp.arg0;
        doorbell_.rsp.arg1 = rsp.arg1;
        doorbell_.rsp.arg2 = rsp.arg2;
        // Kind must be set last, this will trigger the response
        doorbell_.rsp.kind = rsp.kind;
        // Wait until response has been consumed
        while (doorbell_.rsp.kind != Response::Kind::None);
    }
};

} /* namespace bsp */

#endif /* DOORBELL_HPP_ */
