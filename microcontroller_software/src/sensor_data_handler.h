#ifndef SENSOR_DATA_HANDLER_H
#define SENSOR_DATA_HANDLER_H

#define DATA_SEND_UART uart1
#define DATA_SEND_UART_BAUD 9600

#include "pico/stdlib.h"
#include "pico/sync.h"
#include "hardware/uart.h"

namespace SensorDataHandler {

  struct SensorData
  {
    unsigned char hmty[2] = {0, 0};
    unsigned char temp[2] = {0, 0};
    unsigned char time[3] = {0, 0, 0};
  };

  const int MAX_RECORDS = 10000;
  extern SensorData sensorData[MAX_RECORDS];
  extern int sensorDataIndex;
  extern float currentTimeOffset;
  extern int currentReadingDelay;
  //not the array itself, just index or time offsets
  extern critical_section modifyingDataHandlerGlobals;

  void external_rx_handler();
}

#endif
