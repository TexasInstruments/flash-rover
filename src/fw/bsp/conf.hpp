// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

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
