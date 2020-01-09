// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

#ifndef HARD_FAULT_HPP__
#define HARD_FAULT_HPP__

struct RegDump_t
{
    uint32_t r0{ 0 };
    uint32_t r1{ 0 };
    uint32_t r2{ 0 };
    uint32_t r3{ 0 };
    uint32_t r12{ 0 };
    uint32_t lr{ 0 };
    uint32_t pc{ 0 };
    uint32_t psr{ 0 };
};

void openHardFaultDebugger(RegDump_t& regDump);

#endif /* HARD_FAULT_HPP__ */
