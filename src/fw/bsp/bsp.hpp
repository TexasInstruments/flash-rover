
#ifndef __BSP_H__
#define __BSP_H__

#include <stdint.h>

#include <ti/devices/DeviceFamily.h>
#include DeviceFamily_constructPath(inc/hw_types.h)
#include DeviceFamily_constructPath(inc/hw_memmap.h)
#include DeviceFamily_constructPath(inc/hw_sysctl.h) // Access to the GET_MCU_CLOCK define
#include DeviceFamily_constructPath(inc/hw_ioc.h)
#include DeviceFamily_constructPath(driverlib/ioc.h)
#include DeviceFamily_constructPath(driverlib/gpio.h)

// Board LED defines
#define BSP_IOID_LED_1          IOID_6
#define BSP_IOID_LED_2          IOID_7

// Board key defines
#define BSP_IOID_KEY_LEFT       IOID_13
#define BSP_IOID_KEY_RIGHT      IOID_14

// Board external flash defines
#define BSP_IOID_FLASH_CS       IOID_20
#define BSP_SPI_MOSI            IOID_9
#define BSP_SPI_MISO            IOID_8
#define BSP_SPI_CLK_FLASH       IOID_10


#endif /* __BSP_H__ */
