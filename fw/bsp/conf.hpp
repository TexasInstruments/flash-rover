
#ifndef CONF_HPP_
#define CONF_HPP_

#include <stddef.h>
#include <stdint.h>

#include "spi.hpp"

namespace bsp {

struct Conf
{
    uint32_t valid{ 0 };
    SpiPins spiPins{};
} __attribute__((packed));

} /* namespace bsp */

#endif /* CONF_HPP_ */
