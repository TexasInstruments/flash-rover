// Copyright (c) 2020 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

#ifndef POWER_HPP__
#define POWER_HPP__

#include <stddef.h>
#include <stdint.h>

#include <limits>

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/prcm.h)

namespace bsp {

class Power
{
private:
    using dep_count_t = uint8_t;
    static constexpr dep_count_t max_dep_count = std::numeric_limits<dep_count_t>::max();

    struct
    {
        struct
        {
            dep_count_t rfcore{ 0 };
            dep_count_t serial{ 0 };
            dep_count_t periph{ 0 };
            dep_count_t vims{ 0 };
            dep_count_t sysbus{ 0 };
            dep_count_t cpu{ 0 };
        } domains;
        struct
        {
            dep_count_t timer0{ 0 };
            dep_count_t timer1{ 0 };
            dep_count_t timer2{ 0 };
            dep_count_t timer3{ 0 };
            dep_count_t ssi0{ 0 };
            dep_count_t ssi1{ 0 };
            dep_count_t uart0{ 0 };
            dep_count_t uart1{ 0 };
            dep_count_t i2c0{ 0 };
            dep_count_t crypto{ 0 };
            dep_count_t trng{ 0 };
            dep_count_t pka{ 0 };
            dep_count_t udma{ 0 };
            dep_count_t gpio{ 0 };
            dep_count_t i2s{ 0 };
        } periphs;
    } counts_;

public:
    enum class Domain : uint32_t
    {
        RFCore = PRCM_DOMAIN_RFCORE,
        Serial = PRCM_DOMAIN_SERIAL,
        Periph = PRCM_DOMAIN_PERIPH,
        Vims   = PRCM_DOMAIN_VIMS,
        Sysbus = PRCM_DOMAIN_SYSBUS,
        Cpu    = PRCM_DOMAIN_CPU,
        None,
    };

#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X0_CC26X0
    enum class Periph : uint32_t
    {
        Timer0 = PRCM_PERIPH_TIMER0,
        Timer1 = PRCM_PERIPH_TIMER1,
        Timer2 = PRCM_PERIPH_TIMER2,
        Timer3 = PRCM_PERIPH_TIMER3,
        Ssi0   = PRCM_PERIPH_SSI0,
        Ssi1   = PRCM_PERIPH_SSI1,
        Uart0  = PRCM_PERIPH_UART0,
        I2c0   = PRCM_PERIPH_I2C0,
        Crypto = PRCM_PERIPH_CRYPTO,
        Trng   = PRCM_PERIPH_TRNG,
        Udma   = PRCM_PERIPH_UDMA,
        Gpio   = PRCM_PERIPH_GPIO,
        I2s    = PRCM_PERIPH_I2S,
        None,
    };
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X2_CC26X2
    enum class Periph : uint32_t
    {
        Timer0 = PRCM_PERIPH_TIMER0,
        Timer1 = PRCM_PERIPH_TIMER1,
        Timer2 = PRCM_PERIPH_TIMER2,
        Timer3 = PRCM_PERIPH_TIMER3,
        Ssi0   = PRCM_PERIPH_SSI0,
        Ssi1   = PRCM_PERIPH_SSI1,
        Uart0  = PRCM_PERIPH_UART0,
        Uart1  = PRCM_PERIPH_UART1,
        I2c0   = PRCM_PERIPH_I2C0,
        Crypto = PRCM_PERIPH_CRYPTO,
        Trng   = PRCM_PERIPH_TRNG,
        Pka    = PRCM_PERIPH_PKA,
        Udma   = PRCM_PERIPH_UDMA,
        Gpio   = PRCM_PERIPH_GPIO,
        I2s    = PRCM_PERIPH_I2S,
        None,
    };
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X4_CC26X3_CC26X4
    enum class Periph : uint32_t
    {
        Timer0 = PRCM_PERIPH_TIMER0,
        Timer1 = PRCM_PERIPH_TIMER1,
        Timer2 = PRCM_PERIPH_TIMER2,
        Timer3 = PRCM_PERIPH_TIMER3,
        Ssi0   = PRCM_PERIPH_SSI0,
        Ssi1   = PRCM_PERIPH_SSI1,
        Uart0  = PRCM_PERIPH_UART0,
        Uart1  = PRCM_PERIPH_UART1,
        I2c0   = PRCM_PERIPH_I2C0,
        Crypto = PRCM_PERIPH_CRYPTO,
        Trng   = PRCM_PERIPH_TRNG,
        Pka    = PRCM_PERIPH_PKA,
        Udma   = PRCM_PERIPH_UDMA,
        Gpio   = PRCM_PERIPH_GPIO,
        I2s    = PRCM_PERIPH_I2S,
        None,
    };
#endif

    class DomainHandle
    {
        Power& power_;
        Domain domain_;

    public:
        DomainHandle(Power& power, Domain domain)
            : power_{ power }
            , domain_{ domain }
        {
            power_.setDependency(domain_);
        }

        ~DomainHandle()
        {
            power_.clearDependency(domain_);
        }
    };

    class PeriphHandle
    {
        Power& power_;
        Periph periph_;

    public:
        PeriphHandle(Power& power, Periph periph)
            : power_{ power }
            , periph_{ periph }
        {
            power_.setDependency(periph_);
        }

        ~PeriphHandle()
        {
            power_.clearDependency(periph_);
        }
    };


    Power()
    {

    }

    ~Power()
    {

    }

    DomainHandle openDomain(Domain domain)
    {
        return DomainHandle(*this, domain);
    }

    PeriphHandle openPeriph(Periph periph)
    {
        return PeriphHandle(*this, periph);
    }

private:
    Domain getDomainDependency(Periph periph)
    {
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X0_CC26X0
        switch (periph)
        {
        case Periph::Timer0: return Domain::Periph;
        case Periph::Timer1: return Domain::Periph;
        case Periph::Timer2: return Domain::Periph;
        case Periph::Timer3: return Domain::Periph;
        case Periph::Ssi0:   return Domain::Serial;
        case Periph::Ssi1:   return Domain::Periph;
        case Periph::Uart0:  return Domain::Serial;
        case Periph::I2c0:   return Domain::Serial;
        case Periph::Crypto: return Domain::Periph;
        case Periph::Trng:   return Domain::Periph;
        case Periph::Udma:   return Domain::Periph;
        case Periph::Gpio:   return Domain::Periph;
        case Periph::I2s:    return Domain::Periph;
        default:             return Domain::None;
        }
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X2_CC26X2
        switch (periph)
        {
        case Periph::Timer0: return Domain::Periph;
        case Periph::Timer1: return Domain::Periph;
        case Periph::Timer2: return Domain::Periph;
        case Periph::Timer3: return Domain::Periph;
        case Periph::Ssi0:   return Domain::Serial;
        case Periph::Ssi1:   return Domain::Periph;
        case Periph::Uart0:  return Domain::Serial;
        case Periph::Uart1:  return Domain::Periph;
        case Periph::I2c0:   return Domain::Serial;
        case Periph::Crypto: return Domain::Periph;
        case Periph::Trng:   return Domain::Periph;
        case Periph::Pka:    return Domain::Periph;
        case Periph::Udma:   return Domain::Periph;
        case Periph::Gpio:   return Domain::Periph;
        case Periph::I2s:    return Domain::Periph;
        default:             return Domain::None;
        }
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X4_CC26X3_CC26X4
        switch (periph)
        {
        case Periph::Timer0: return Domain::Periph;
        case Periph::Timer1: return Domain::Periph;
        case Periph::Timer2: return Domain::Periph;
        case Periph::Timer3: return Domain::Periph;
        case Periph::Ssi0:   return Domain::Serial;
        case Periph::Ssi1:   return Domain::Periph;
        case Periph::Uart0:  return Domain::Serial;
        case Periph::Uart1:  return Domain::Periph;
        case Periph::I2c0:   return Domain::Serial;
        case Periph::Crypto: return Domain::Periph;
        case Periph::Trng:   return Domain::Periph;
        case Periph::Pka:    return Domain::Periph;
        case Periph::Udma:   return Domain::Periph;
        case Periph::Gpio:   return Domain::Periph;
        case Periph::I2s:    return Domain::Periph;
        default:             return Domain::None;
        }
#endif
    }

    dep_count_t* getDepCount(Domain domain)
    {
        switch (domain)
        {
        case Domain::RFCore: return &counts_.domains.rfcore;
        case Domain::Serial: return &counts_.domains.serial;
        case Domain::Periph: return &counts_.domains.periph;
        case Domain::Vims:   return &counts_.domains.vims;
        case Domain::Sysbus: return &counts_.domains.sysbus;
        case Domain::Cpu:    return &counts_.domains.cpu;
        default:             return nullptr;
        }
    }

    dep_count_t* getDepCount(Periph periph)
    {
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X0_CC26X0
        switch (periph)
        {
        case Periph::Timer0: return &counts_.periphs.timer0;
        case Periph::Timer1: return &counts_.periphs.timer1;
        case Periph::Timer2: return &counts_.periphs.timer2;
        case Periph::Timer3: return &counts_.periphs.timer3;
        case Periph::Ssi0:   return &counts_.periphs.ssi0;
        case Periph::Ssi1:   return &counts_.periphs.ssi1;
        case Periph::Uart0:  return &counts_.periphs.uart0;
        case Periph::I2c0:   return &counts_.periphs.i2c0;
        case Periph::Crypto: return &counts_.periphs.crypto;
        case Periph::Trng:   return &counts_.periphs.trng;
        case Periph::Udma:   return &counts_.periphs.udma;
        case Periph::Gpio:   return &counts_.periphs.gpio;
        case Periph::I2s:    return &counts_.periphs.i2s;
        default:             return nullptr;
        }
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X2_CC26X2
        switch (periph)
        {
        case Periph::Timer0: return &counts_.periphs.timer0;
        case Periph::Timer1: return &counts_.periphs.timer1;
        case Periph::Timer2: return &counts_.periphs.timer2;
        case Periph::Timer3: return &counts_.periphs.timer3;
        case Periph::Ssi0:   return &counts_.periphs.ssi0;
        case Periph::Ssi1:   return &counts_.periphs.ssi1;
        case Periph::Uart0:  return &counts_.periphs.uart0;
        case Periph::Uart1:  return &counts_.periphs.uart1;
        case Periph::I2c0:   return &counts_.periphs.i2c0;
        case Periph::Crypto: return &counts_.periphs.crypto;
        case Periph::Trng:   return &counts_.periphs.trng;
        case Periph::Pka:    return &counts_.periphs.pka;
        case Periph::Udma:   return &counts_.periphs.udma;
        case Periph::Gpio:   return &counts_.periphs.gpio;
        case Periph::I2s:    return &counts_.periphs.i2s;
        default:             return nullptr;
        }
#endif
#if DeviceFamily_PARENT == DeviceFamily_PARENT_CC13X4_CC26X3_CC26X4
        switch (periph)
        {
        case Periph::Timer0: return &counts_.periphs.timer0;
        case Periph::Timer1: return &counts_.periphs.timer1;
        case Periph::Timer2: return &counts_.periphs.timer2;
        case Periph::Timer3: return &counts_.periphs.timer3;
        case Periph::Ssi0:   return &counts_.periphs.ssi0;
        case Periph::Ssi1:   return &counts_.periphs.ssi1;
        case Periph::Uart0:  return &counts_.periphs.uart0;
        case Periph::Uart1:  return &counts_.periphs.uart1;
        case Periph::I2c0:   return &counts_.periphs.i2c0;
        case Periph::Crypto: return &counts_.periphs.crypto;
        case Periph::Trng:   return &counts_.periphs.trng;
        case Periph::Pka:    return &counts_.periphs.pka;
        case Periph::Udma:   return &counts_.periphs.udma;
        case Periph::Gpio:   return &counts_.periphs.gpio;
        case Periph::I2s:    return &counts_.periphs.i2s;
        default:             return nullptr;
        }
#endif
    }

    void setDependency(Domain domain)
    {
        auto maybe_dep_count = getDepCount(domain);
        if (maybe_dep_count == nullptr)
        {
            return;
        }

        auto& dep_count = *maybe_dep_count;

        if (dep_count == max_dep_count)
        {
            return;
        }

        dep_count += 1;
        if (dep_count == 1)
        {
            uint32_t u32domain = static_cast<uint32_t>(domain);
            PRCMPowerDomainOn(u32domain);

            while (PRCMPowerDomainsAllOn(u32domain) != PRCM_DOMAIN_POWER_ON);
        }
    }

    void clearDependency(Domain domain)
    {
        auto maybe_dep_count = getDepCount(domain);
        if (maybe_dep_count == nullptr)
        {
            return;
        }

        auto& dep_count = *maybe_dep_count;

        if (dep_count == 0)
        {
            return;
        }

        dep_count -= 1;
        if (dep_count == 0)
        {
            uint32_t u32domain = static_cast<uint32_t>(domain);
            PRCMPowerDomainOff(u32domain);
            while (PRCMPowerDomainsAllOff(u32domain) != PRCM_DOMAIN_POWER_OFF);
        }
    }

    void setDependency(Periph periph)
    {
        auto maybe_dep_count = getDepCount(periph);
        if (maybe_dep_count == nullptr)
        {
            return;
        }

        auto& dep_count = *maybe_dep_count;

        if (dep_count == max_dep_count)
        {
            return;
        }

        dep_count += 1;
        if (dep_count == 1)
        {
            Domain parent = getDomainDependency(periph);
            setDependency(parent);

            uint32_t u32periph = static_cast<uint32_t>(periph);
            PRCMPeripheralRunEnable(u32periph);
            PRCMLoadSet();
            while (!PRCMLoadGet());
        }
    }

    void clearDependency(Periph periph)
    {
        auto maybe_dep_count = getDepCount(periph);
        if (maybe_dep_count == nullptr)
        {
            return;
        }

        auto& dep_count = *maybe_dep_count;

        if (dep_count == 0)
        {
            return;
        }

        dep_count -= 1;
        if (dep_count == 0)
        {
            uint32_t u32periph = static_cast<uint32_t>(periph);
            PRCMPeripheralRunDisable(u32periph);
            PRCMLoadSet();
            while (!PRCMLoadGet());

            Domain parent = getDomainDependency(periph);
            clearDependency(parent);
        }
    }
};

} /* namespace bsp */

#endif /* POWER_HPP__ */
