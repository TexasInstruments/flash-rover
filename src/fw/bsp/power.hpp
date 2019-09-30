
#ifndef POWER_HPP__
#define POWER_HPP__

#include <stddef.h>
#include <stdint.h>

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(driverlib/prcm.h)

namespace bsp {

class Power
{
private:
    using dep_count_t = uint8_t;
    static constexpr dep_count_t max_dep_count = static_cast<dep_count_t>(~0);

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
    }

    void setDependency(Domain domain)
    {
        auto inc_and_power_on = [&](dep_count_t &dep_count)
        {
            if (dep_count == max_dep_count)
            {
                return;
            }

            dep_count += 1;
            if (dep_count == 1)
            {
                uint32_t u32domain = static_cast<uint32_t>(domain);
                PRCMPowerDomainOn(u32domain);
                while (PRCMPowerDomainStatus(u32domain) != PRCM_DOMAIN_POWER_ON);
            }
        };

        switch (domain)
        {
        case Domain::RFCore: inc_and_power_on(counts_.domains.rfcore); break;
        case Domain::Serial: inc_and_power_on(counts_.domains.serial); break;
        case Domain::Periph: inc_and_power_on(counts_.domains.periph); break;
        case Domain::Vims:   inc_and_power_on(counts_.domains.vims);   break;
        case Domain::Sysbus: inc_and_power_on(counts_.domains.sysbus); break;
        case Domain::Cpu:    inc_and_power_on(counts_.domains.cpu);    break;
        default:             /* do nothing */                          break;
        }
    }

    void clearDependency(Domain domain)
    {
        auto dec_and_power_off = [&](dep_count_t &dep_count)
        {
            if (dep_count == 0)
            {
                return;
            }

            dep_count -= 1;
            if (dep_count == 0)
            {
                uint32_t u32domain = static_cast<uint32_t>(domain);
                PRCMPowerDomainOff(u32domain);
                while (PRCMPowerDomainStatus(u32domain) != PRCM_DOMAIN_POWER_OFF);
            }
        };

        switch (domain)
        {
        case Domain::RFCore: dec_and_power_off(counts_.domains.rfcore); break;
        case Domain::Serial: dec_and_power_off(counts_.domains.serial); break;
        case Domain::Periph: dec_and_power_off(counts_.domains.periph); break;
        case Domain::Vims:   dec_and_power_off(counts_.domains.vims);   break;
        case Domain::Sysbus: dec_and_power_off(counts_.domains.sysbus); break;
        case Domain::Cpu:    dec_and_power_off(counts_.domains.cpu);    break;
        default:             /* do nothing */                           break;
        }
    }

    void setDependency(Periph periph)
    {
        auto inc_and_power_on = [&](dep_count_t &dep_count)
        {
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
        };

        switch (periph)
        {
        case Periph::Timer0: inc_and_power_on(counts_.periphs.timer0); break;
        case Periph::Timer1: inc_and_power_on(counts_.periphs.timer1); break;
        case Periph::Timer2: inc_and_power_on(counts_.periphs.timer2); break;
        case Periph::Timer3: inc_and_power_on(counts_.periphs.timer3); break;
        case Periph::Ssi0:   inc_and_power_on(counts_.periphs.ssi0);   break;
        case Periph::Ssi1:   inc_and_power_on(counts_.periphs.ssi1);   break;
        case Periph::Uart0:  inc_and_power_on(counts_.periphs.uart0);  break;
        case Periph::Uart1:  inc_and_power_on(counts_.periphs.uart1);  break;
        case Periph::I2c0:   inc_and_power_on(counts_.periphs.i2c0);   break;
        case Periph::Crypto: inc_and_power_on(counts_.periphs.crypto); break;
        case Periph::Trng:   inc_and_power_on(counts_.periphs.trng);   break;
        case Periph::Pka:    inc_and_power_on(counts_.periphs.pka);    break;
        case Periph::Udma:   inc_and_power_on(counts_.periphs.udma);   break;
        case Periph::Gpio:   inc_and_power_on(counts_.periphs.gpio);   break;
        case Periph::I2s:    inc_and_power_on(counts_.periphs.i2s);    break;
        default:              /* do nothing */                         break;
        }
    }

    void clearDependency(Periph periph)
    {
        auto dec_and_power_off = [&](dep_count_t &dep_count)
        {
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
        };

        switch (periph)
        {
        case Periph::Timer0: dec_and_power_off(counts_.periphs.timer0); break;
        case Periph::Timer1: dec_and_power_off(counts_.periphs.timer1); break;
        case Periph::Timer2: dec_and_power_off(counts_.periphs.timer2); break;
        case Periph::Timer3: dec_and_power_off(counts_.periphs.timer3); break;
        case Periph::Ssi0:   dec_and_power_off(counts_.periphs.ssi0);   break;
        case Periph::Ssi1:   dec_and_power_off(counts_.periphs.ssi1);   break;
        case Periph::Uart0:  dec_and_power_off(counts_.periphs.uart0);  break;
        case Periph::Uart1:  dec_and_power_off(counts_.periphs.uart1);  break;
        case Periph::I2c0:   dec_and_power_off(counts_.periphs.i2c0);   break;
        case Periph::Crypto: dec_and_power_off(counts_.periphs.crypto); break;
        case Periph::Trng:   dec_and_power_off(counts_.periphs.trng);   break;
        case Periph::Pka:    dec_and_power_off(counts_.periphs.pka);    break;
        case Periph::Udma:   dec_and_power_off(counts_.periphs.udma);   break;
        case Periph::Gpio:   dec_and_power_off(counts_.periphs.gpio);   break;
        case Periph::I2s:    dec_and_power_off(counts_.periphs.i2s);    break;
        default:              /* do nothing */                           break;
        }
    }
};

} /* namespace bsp */

#endif /* POWER_HPP__ */
