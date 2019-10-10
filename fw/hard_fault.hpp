// This file is covered by the LICENSE file in the root of this project.

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
