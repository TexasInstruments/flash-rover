
#ifndef HARD_FAULT_HPP__
#define HARD_FAULT_HPP__

struct RegDump_t
{
    uint32_t r0;
    uint32_t r1;
    uint32_t r2;
    uint32_t r3;
    uint32_t r12;
    uint32_t lr;
    uint32_t pc;
    uint32_t psr;
};

void openHardFaultDebugger(RegDump_t& regDump);

#endif /* HARD_FAULT_HPP__ */
